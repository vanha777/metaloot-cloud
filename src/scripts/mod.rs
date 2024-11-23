pub fn get_user_script(address: String) -> serde_json::Value {
    let script = r#"
import NonFungibleToken from 0x631e88ae7f1d7c20
import MetaLootNFT from 0xceed54f46d4b1942

// Return the NFTs owned by account 0x01.
access(all)
fun main(address: Address): [&{NonFungibleToken.NFT}] {
    // Get the public account object for the specified address
    let nftOwner = getAccount(address)

    // Find the public Receiver capability for their Collection and borrow it
    let collectionRef = nftOwner.capabilities.borrow<&{NonFungibleToken.Collection}>(
            MetaLootNFT.CollectionPublicPath
    ) ?? panic("The account ".concat(address.toString()).concat(" does not have a NonFungibleToken Collection at ")
                .concat(MetaLootNFT.CollectionPublicPath.toString())
                .concat(". The account must initialize their account with this collection first!"))

    // Get the IDs of the NFTs owned by the account
    let nftIDs = collectionRef.getIDs()

    // Create an array to store references to the NFTs
    var nfts: [&{NonFungibleToken.NFT}] = []

    // Iterate over the IDs and get each NFT reference
    for id in nftIDs {
        let nftRef: &{NonFungibleToken.NFT} = collectionRef.borrowNFT(id)
            ?? panic("Could not borrow NFT with id ".concat(id.toString()))
        nfts.append(nftRef)
    }
    return nfts
}
    "#;
    let arguments = format!(r#"{{"type":"Address","value":"{}"}}"#, address);
    let script_base64 = base64::encode(script);
    let arguments_base64 = base64::encode(arguments);
    serde_json::json!({
        "script": script_base64,
        "arguments": [
            arguments_base64
        ]
    })
}
