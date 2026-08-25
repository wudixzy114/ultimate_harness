//! Peer management — keep track of connected WebSocket clients.

use std::sync::Arc;

use serde_json::{Value, json};
use tokio::sync::{Mutex, mpsc};
use uuid::Uuid;

use crate::state::PeerHandle;

pub type Peers = Arc<Mutex<Vec<PeerHandle>>>;

/// Add a new peer. Returns its id and a sender that the WS task can
/// use to push events.
pub async fn add_peer(peers: &Peers) -> Uuid {
    let id = Uuid::new_v4();
    let (tx, _rx) = mpsc::unbounded_channel::<Value>();
    let handle = PeerHandle { id, sender: tx };
    peers.lock().await.push(handle);
    id
}

pub async fn remove_peer(peers: &Peers, id: Uuid) {
    let mut g = peers.lock().await;
    g.retain(|p| p.id != id);
}

/// Broadcast a JSON event to all connected peers.
pub async fn broadcast(peers: &Peers, event: &str, data: Value) {
    let payload = json!({
        "kind": "event",
        "event": event,
        "data": data,
    });
    let g = peers.lock().await;
    for p in g.iter() {
        let _ = p.sender.send(payload.clone());
    }
}
