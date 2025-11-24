// src-tauri/src/commands/blockchain_commands.rs
//! Commandes Tauri liées à la Blockchain et au VPN mesh.
//!
//! ⚠️ IMPORTANT :
//! - On ne modifie PAS le module `crate::blockchain`.
//! - On s'appuie uniquement sur les types et méthodes déjà exposés par ce module
//!   (par ex. `FabricClient`, `InnernetClient`, éventuellement `ping()`, `status()`, etc.).
//! - Toute la logique "métier" côté commandes est ici, avec des stubs simples
//!   que l'on pourra enrichir plus tard.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::blockchain::{FabricClient, InnernetClient};

/// Résultat générique d'une transaction Fabric exposé au frontend.
#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionResult {
    /// OK / KO.
    pub success: bool,
    /// Message humain (erreur, info, etc.).
    pub message: String,
    /// Charge utile optionnelle (JSON).
    #[serde(default)]
    pub payload: Option<serde_json::Value>,
}

/// Représente un peer Innernet (ou un nœud du mesh VPN).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    /// Nom logique du peer (ex: "peer0.org1").
    pub name: String,
    /// Adresse (IP ou hostname).
    pub address: String,
    /// Est-ce que le peer est vu comme en ligne ?
    pub online: bool,
}

/// État synthétique du réseau Innernet.
#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkStatus {
    /// Profil Innernet utilisé (ex: "dev", "prod").
    pub profile: String,
    /// Est-ce que le VPN est considéré comme connecté ?
    pub connected: bool,
    /// Liste des peers visibles.
    pub peers: Vec<Peer>,
}

// -----------------------------------------------------------------------------
// Commandes Fabric
// -----------------------------------------------------------------------------

/// Ping Fabric pour vérifier la configuration du client côté backend.
#[tauri::command]
pub fn fabric_ping(client: State<FabricClient>) -> String {
    // On suppose que `FabricClient` expose une méthode `ping()`.
    // Si ce n'est pas le cas, tu peux simplement remplacer par un message fixe.
    client.ping()
}

/// Stub d'envoi de transaction Fabric.
/// On ne dépend d'aucune méthode spécifique de `FabricClient`
/// (comme `submit_transaction`), pour respecter le module `blockchain`.
#[tauri::command]
pub fn fabric_submit_transaction(
    client: State<FabricClient>,
    chaincode: String,
    function: String,
    args: Vec<String>,
) -> TransactionResult {
    // Ici on construit simplement un résultat "stub" qui sera visible côté frontend.
    // Plus tard, tu pourras remplacer ce code par un appel à ton vrai client Fabric.
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

/// Stub de requête Fabric (lecture).
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

/// Stub de récupération d'historique pour une clé donnée.
#[tauri::command]
pub fn fabric_get_history(client: State<FabricClient>, key: String) -> TransactionResult {
    let connection_info = client.ping();

    TransactionResult {
        success: true,
        message: format!("stub get_history: key={key}, conn={}", connection_info),
        payload: None,
    }
}

// -----------------------------------------------------------------------------
// Commandes VPN / Innernet
// -----------------------------------------------------------------------------

/// Retourne un état synthétique du VPN mesh.
/// On s'appuie uniquement sur ce que fournit `InnernetClient`
/// (par ex. une méthode `status()` ou un champ `profile`).
#[tauri::command]
pub fn vpn_network_status(client: State<InnernetClient>) -> NetworkStatus {
    // On suppose que `InnernetClient` expose au moins :
    // - un champ `profile: String` (ou équivalent),
    // - une méthode `status(&self) -> String`.
    let status = client.status();

    NetworkStatus {
        profile: client.profile.clone(),
        // Ici on "devine" un booléen à partir du status texte (stub).
        connected: !status.is_empty(),
        peers: Vec::new(), // pour l'instant, aucun peer n'est remonté.
    }
}

/// Stub de "connexion" au VPN.
/// Ne dépend d'aucune méthode spécifique (`connect()`, etc.) sur `InnernetClient`.
#[tauri::command]
pub fn vpn_connect(_client: State<InnernetClient>) -> Result<(), String> {
    // Plus tard : appeler ici la vraie logique (binaire innernet, etc.)
    // Pour l'instant : succès systématique.
    Ok(())
}

/// Stub de "déconnexion" du VPN.
#[tauri::command]
pub fn vpn_disconnect(_client: State<InnernetClient>) -> Result<(), String> {
    Ok(())
}

/// Liste les peers connus (stub : renvoie une liste vide).
#[tauri::command]
pub fn vpn_list_peers(_client: State<InnernetClient>) -> Vec<Peer> {
    Vec::new()
}

/// Ajoute un peer (stub : ne fait que valider les paramètres).
#[tauri::command]
pub fn vpn_add_peer(_client: State<InnernetClient>, peer: Peer) -> Result<(), String> {
    // Ici on pourrait logguer ou persister ce peer plus tard.
    println!("vpn_add_peer stub: {} ({})", peer.name, peer.address);
    Ok(())
}

/// Ping un peer (stub).
#[tauri::command]
pub fn vpn_ping_peer(_client: State<InnernetClient>, peer: Peer) -> Result<(), String> {
    println!("vpn_ping_peer stub: {} ({})", peer.name, peer.address);
    Ok(())
}

/// Vérifie (de façon très simplifiée) l'installation d'Innernet.
/// Ici, on ne dépend pas d'une méthode associée `InnernetClient::check_installation`.
#[tauri::command]
pub fn vpn_check_installation() -> bool {
    // Plus tard : vérifier présence du binaire innernet, version, etc.
    true
}
