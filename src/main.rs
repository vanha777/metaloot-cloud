use std::{collections::HashMap, sync::Arc, time::Duration};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::{get, post},
    Extension, Router,
};
use chrono::{DateTime, Utc};
use futures::{stream::SplitSink, SinkExt, StreamExt};
use model::FlattenedItem;
use serde::{Deserialize, Serialize};
use shuttle_axum::ShuttleAxum;
use tokio::{
    sync::{watch, Mutex},
    time::sleep,
};
use tower_http::services::ServeDir;
pub mod handler;
pub mod model;
pub mod scripts;
struct State {
    total_clients: usize,
    connections: HashMap<String, Arc<Mutex<SplitSink<WebSocket, Message>>>>,
    rx: watch::Receiver<Message>,
}

const PAUSE_SECS: u64 = 15;
const STATUS_URI: &str = "https://api.shuttle.dev/.healthz";

#[derive(Debug, Serialize, Deserialize)]
struct Response {
    data: Option<Vec<FlattenedItem>>,
    total_clients: usize,
    #[serde(rename = "dateTime")]
    date_time: DateTime<Utc>,
    result: bool,
}

#[shuttle_runtime::main]
async fn main() -> ShuttleAxum {
    let (tx, rx) = watch::channel(Message::Text("{}".to_string()));

    let state = Arc::new(Mutex::new(State {
        total_clients: 0,
        connections: HashMap::new(),
        rx,
    }));

    // Spawn a thread to continually check the status of the api
    // let state_send: Arc<Mutex<State>> = state.clone();
    // tokio::spawn(async move {
    //     let duration = Duration::from_secs(PAUSE_SECS);

    //     loop {
    //         // let result = reqwest::get(STATUS_URI)
    //         //     .await
    //         //     .is_ok_and(|r| r.status().is_success());

    //         let response = Response {
    //             total_clients: state_send.lock().await.total_clients,
    //             date_time: Utc::now(),
    //             result:false
    //         };
    //         let msg = serde_json::to_string(&response).unwrap();

    //         if tx.send(Message::Text(msg)).is_err() {
    //             break;
    //         }

    //         sleep(duration).await;
    //     }
    // });

    let router = Router::new()
        .route("/websocket/:address", get(websocket_handler))
        .route("/api/user/:address_id", get(handler::get_user))
        .route("/api/:address_id/game/start", get(handler::game_start))
        .route("/api/:address_id/mint", post(handler::mint_nft))
        .route("/api/:address_id/game/end", get(handler::game_end))
        .nest_service("/", ServeDir::new("static"))
        .layer(Extension(state))
        .layer(tower_http::cors::CorsLayer::new()
            .allow_origin(tower_http::cors::Any)
            .allow_methods(tower_http::cors::Any)
            .allow_headers(tower_http::cors::Any));
    Ok(router.into())
}

async fn websocket_handler(
    Path(address): Path<String>,
    ws: WebSocketUpgrade,
    Extension(state): Extension<Arc<Mutex<State>>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket(address, socket, state))
}

async fn websocket(address_id: String, stream: WebSocket, state: Arc<Mutex<State>>) {
    // Split the WebSocket stream into sender and receiver
    let (sender, mut receiver) = stream.split();

    // Clone the watch receiver for broadcasting
    // let mut rx = {
    //     let mut state = state.lock().await;
    //     state.total_clients += 1;
    //     // Insert the sender into the connections HashMap wrapped in Arc<Mutex<>>
    //     state
    //         .connections
    //         .insert(address_id.clone(), Arc::new(Mutex::new(sender)));
    //     state.rx.clone()
    // };

    let mut rx = {
        let mut state = state.lock().await;

        // Check if the connection for the address_id already exists
        if state.connections.contains_key(&address_id) {
            return eprintln!("Connection for address_id '{}' already exists", address_id);
        } else {
            // Insert the sender into the connections HashMap wrapped in Arc<Mutex<>>
            state.total_clients += 1;
            state
                .connections
                .insert(address_id.clone(), Arc::new(Mutex::new(sender)));
            state.rx.clone()
        }
    };

    // Clone the connection Arc for the send task
    // let connection = {
    //     let state = state.lock().await;
    //     state.connections.get(&address_id).unwrap().clone()
    // };

    // Spawn a task to send broadcast messages to the client
    // let mut send_task = tokio::spawn(async move {
    //     while let Ok(()) = rx.changed().await {
    //         let msg = rx.borrow().clone();
    //         let mut sender = connection.lock().await;
    //         if sender.send(msg.clone()).await.is_err() {
    //             break;
    //         }
    //     }
    // });

    // Spawn a task to receive messages from the client
    // Clone address_id before moving it into the task
    let address_id_clone = address_id.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            println!("Received message from {}: {}", address_id_clone, text);
            // Handle incoming messages here if needed
        }
    });

    // Use tokio::select! with mutable references
    // tokio::select! {
    //     _ = &mut send_task => recv_task.abort(),
    //     _ = &mut recv_task => send_task.abort(),
    // };

    // Await the recv_task to complete
    if let Err(e) = recv_task.await {
        eprintln!("recv_task failed for {}: {}", address_id, e);
    }

    // Clean up when the client disconnects
    {
        let mut state = state.lock().await;
        state.total_clients -= 1;
        state.connections.remove(&address_id);
    }

    println!("WebSocket connection with {} closed", address_id);
}
