// src-tauri/src/commands/blockchain_commands.rs
//! Commandes Tauri liées à la Blockchain et au VPN mesh.

use serde::{Deserialize, Serialize};
use tauri::State;

// CORRECTION : Utilisation de crate:: car nous sommes dans la lib
use crate::blockchain::{FabricClient, InnernetClient};

/// Résultat générique d'une transaction Fabric exposé au frontend.
#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionResult {
    pub success: bool,
    pub message: String,
    #[serde(default)]
    pub payload: Option<serde_json::Value>,
}

/// Représente un peer Innernet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub name: String,
    pub address: String,
    pub online: bool,
}

/// État synthétique du réseau Innernet.
#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkStatus {
    pub profile: String,
    pub connected: bool,
    pub peers: Vec<Peer>,
}

#[tauri::command]
pub fn fabric_ping(client: State<FabricClient>) -> String {
    client.ping()
}

#[tauri::command]
pub fn fabric_submit_transaction(
    client: State<FabricClient>,
    chaincode: String,
    function: String,
    args: Vec<String>,
) -> TransactionResult {
    let connection_info = client.ping();
    TransactionResult {
        success: true,
        message: format!(
            "stub submit_transaction: chaincode={chaincode}, fn={function}, args={:?}, conn={}",
            args, connection_info
        ),
        payload: None,
    }
}

#[tauri::command]
pub fn fabric_query_transaction(
    client: State<FabricClient>,
    chaincode: String,
    function: String,
    args: Vec<String>,
) -> TransactionResult {
    let connection_info = client.ping();
    TransactionResult {
        success: true,
        message: format!(
            "stub query_transaction: chaincode={chaincode}, fn={function}, args={:?}, conn={}",
            args, connection_info
        ),
        payload: None,
    }
}

#[tauri::command]
pub fn fabric_get_history(client: State<FabricClient>, key: String) -> TransactionResult {
    let connection_info = client.ping();
    TransactionResult {
        success: true,
        message: format!("stub get_history: key={key}, conn={}", connection_info),
        payload: None,
    }
}

#[tauri::command]
pub fn vpn_network_status(client: State<InnernetClient>) -> NetworkStatus {
    let status = client.status();
    NetworkStatus {
        profile: client.profile.clone(),
        connected: !status.is_empty(),
        peers: Vec::new(),
    }
}

#[tauri::command]
pub fn vpn_connect(_client: State<InnernetClient>) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub fn vpn_disconnect(_client: State<InnernetClient>) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub fn vpn_list_peers(_client: State<InnernetClient>) -> Vec<Peer> {
    Vec::new()
}

#[tauri::command]
pub fn vpn_add_peer(_client: State<InnernetClient>, peer: Peer) -> Result<(), String> {
    println!("vpn_add_peer stub: {} ({})", peer.name, peer.address);
    Ok(())
}

#[tauri::command]
pub fn vpn_ping_peer(_client: State<InnernetClient>, peer: Peer) -> Result<(), String> {
    println!("vpn_ping_peer stub: {} ({})", peer.name, peer.address);
    Ok(())
}

#[tauri::command]
pub fn vpn_check_installation() -> bool {
    true
}
