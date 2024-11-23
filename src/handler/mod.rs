use crate::{model::{NFTItem, Root}, scripts::get_user_script, Response, State};
use axum::{
    extract::{ws::Message, Extension, Path},
    response::IntoResponse,
    Json,
};
use base64::{engine::general_purpose, Engine};
use futures::SinkExt;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn get_user(
    Path(address_id): Path<String>,
    Extension(state): Extension<Arc<Mutex<State>>>,
) -> impl IntoResponse {
    println!("debug 0");
    let state = state.lock().await;
    // Make POST request to Flow testnet
    let client = reqwest::Client::new();
    let response = client
        .post("https://rest-testnet.onflow.org/v1/scripts")
        .json(&get_user_script(address_id))
        .send()
        .await;
    println!("debug 1");
    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.text_with_charset("utf-8").await {
                    Ok(data) => {
                        // Decode Base64
                        let data = data.trim_matches('"');
                        let decoded = match general_purpose::STANDARD.decode(data.trim()) {
                            Ok(decoded) => decoded,
                            Err(e) => {
                                eprintln!("Failed to decode Base64: {}", e);
                                return Json(Response {
                                    total_clients: state.total_clients,
                                    date_time: chrono::Utc::now(),
                                    result: false,
                                    data: None,
                                });
                            }
                        };
                        // Convert to String
                        let json_str = match String::from_utf8(decoded) {
                            Ok(json_str) => json_str,
                            Err(e) => {
                                eprintln!("Failed to convert to UTF-8: {}", e);
                                return Json(Response {
                                    total_clients: state.total_clients,
                                    date_time: chrono::Utc::now(),
                                    result: false,
                                    data: None
                                });
                            }
                        };
                        let root: Root = serde_json::from_str(&json_str).unwrap();
                        let mut items = Vec::new();
                        for item in root.value {
                            let flattened = item.value.flatten();
                            items.push(flattened);
                            // println!("{:#?}", flattened);
                        }
                        Json(Response {
                            total_clients: state.total_clients,
                            date_time: chrono::Utc::now(),
                            result: true,
                            data: Some(items)
                        })
                    }
                    Err(_e) => Json(Response {
                        total_clients: state.total_clients,
                        date_time: chrono::Utc::now(),
                        result: false,
                        data: None,
                    }),
                }
            } else {
                println!("debug 2 {:?}", resp.text().await.unwrap());
                Json(Response {
                    total_clients: state.total_clients,
                    date_time: chrono::Utc::now(),
                    result: false,
                    data: None
                })
            }
        }
        Err(_) => Json(Response {
            total_clients: state.total_clients,
            date_time: chrono::Utc::now(),
            result: false,
            data: None
        }),
    }
}

pub async fn game_start(
    Path(address_id): Path<String>,
    Extension(state): Extension<Arc<Mutex<State>>>,
) -> impl IntoResponse {
    let state = state.lock().await;
    println!("trying ws ....... ");
    // Find the WebSocket connection for this address
    if let Some(ws_sender) = state.connections.get(&address_id) {
        println!("found ws address ....... ");
        let mut sender = ws_sender.lock().await;

        // Send a message to request user data
        if let Ok(_) = sender.send(Message::Text("game-start".to_string())).await {
            return Json(Response {
                total_clients: state.total_clients,
                date_time: chrono::Utc::now(),
                result: true,
                data: None
            });
        }
    }

    // Return error response if connection not found or send failed
    Json(Response {
        total_clients: state.total_clients,
        date_time: chrono::Utc::now(),
        result: false,
        data: None
    })
}

pub async fn game_end(
    Path(address_id): Path<String>,
    Extension(state): Extension<Arc<Mutex<State>>>,
) -> impl IntoResponse {
    let state = state.lock().await;
    println!("trying ws ....... ");
    // Find the WebSocket connection for this address
    if let Some(ws_sender) = state.connections.get(&address_id) {
        println!("found ws address ....... ");
        let mut sender = ws_sender.lock().await;

        // Send a message to request user data
        if let Ok(_) = sender.send(Message::Text("game-end".to_string())).await {
            return Json(Response {
                total_clients: state.total_clients,
                date_time: chrono::Utc::now(),
                result: true,
                data: None
            });
        }
    }

    // Return error response if connection not found or send failed
    Json(Response {
        total_clients: state.total_clients,
        date_time: chrono::Utc::now(),
        result: false,
        data: None
    })
}

pub async fn mint_nft(
    Path(address_id): Path<String>,
    Extension(state): Extension<Arc<Mutex<State>>>,
    Json(request): Json<NFTItem>,
) -> impl IntoResponse {
    let state = state.lock().await;
    println!("trying ws ....... ");
    // Find the WebSocket connection for this address
    if let Some(ws_sender) = state.connections.get(&address_id) {
        println!("found ws address ....... ");
        let mut sender = ws_sender.lock().await;

        // Send a message to request user data
        let message = serde_json::json!({
            "type": "mint-nft",
            "data": request
        });
        if let Ok(_) = sender.send(Message::Text(message.to_string())).await {
            return Json(Response {
                total_clients: state.total_clients,
                date_time: chrono::Utc::now(),
                result: true,
                data: None
            });
        }
    }

    // Return error response if connection not found or send failed
    Json(Response {
        total_clients: state.total_clients,
        date_time: chrono::Utc::now(),
        result: false,
        data: None
    })
}
