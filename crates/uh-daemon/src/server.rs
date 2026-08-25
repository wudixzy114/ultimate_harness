//! Axum server: HTTP (web static + REST) + WebSocket.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use futures::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use uh_core::transport::WsMessage;

use crate::router::dispatch;
use crate::state::{AppState, PeerHandle};

pub async fn serve(state: Arc<AppState>, addr: SocketAddr) -> anyhow::Result<()> {
    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!(%addr, "uh-daemon listening");
    axum::serve(listener, app).await?;
    Ok(())
}

pub fn build_router(state: Arc<AppState>) -> Router {
    let api = Router::new()
        .route("/api/health", get(health))
        .route("/api/llm", get(llm_info))
        .route("/api/tools", get(tools_list))
        .route("/ws", get(ws_handler));

    // Serve web/dist if built. We look at ../apps/web/dist relative to
    // the daemon's CWD. The user can also run `pnpm dev` separately
    // and the Vite dev server will take over.
    let web_dist = std::path::Path::new("../apps/web/dist");
    if web_dist.is_dir() {
        let serve = tower_http::services::ServeDir::new(web_dist)
            .fallback(tower_http::services::ServeFile::new(web_dist.join("index.html")));
        Router::new()
            .route("/api/health", get(health))
            .route("/api/llm", get(llm_info))
            .route("/api/tools", get(tools_list))
            .route("/ws", get(ws_handler))
            .fallback_service(serve)
            .with_state(state)
    } else {
        api.with_state(state)
    }
}

async fn health() -> impl IntoResponse {
    json_response(json!({ "ok": true }))
}

async fn llm_info(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    json_response(json!({
        "provider": state.llm.name(),
        "model": state.llm.model(),
    }))
}

async fn tools_list(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let specs = if state.config.tools.enable.is_empty() {
        state.tools.specs()
    } else {
        state.tools.specs_filtered(&state.config.tools.enable)
    };
    json_response(json!({ "tools": specs }))
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, state))
}

async fn handle_ws(socket: WebSocket, state: Arc<AppState>) {
    let peer_id = crate::peers::add_peer(&state.peers).await;
    info!(%peer_id, "ws connected");

    let (mut ws_tx, mut ws_rx) = socket.split();
    let (out_tx, mut out_rx) = mpsc::unbounded_channel::<Value>();

    // Background: drain out_rx and write to the WS.
    let write_task = tokio::spawn(async move {
        while let Some(v) = out_rx.recv().await {
            let text = match serde_json::to_string(&v) {
                Ok(s) => s,
                Err(e) => {
                    error!(error = %e, "encode ws message");
                    continue;
                }
            };
            if ws_tx.send(Message::Text(text)).await.is_err() {
                break;
            }
        }
    });

    // Reader: accept frames, dispatch, send responses via out_tx.
    let state2 = Arc::clone(&state);
    let out_tx_for_read = out_tx.clone();
    let read_task: tokio::task::JoinHandle<()> = tokio::spawn(async move {
        let out_tx = out_tx_for_read;
        while let Some(msg) = ws_rx.next().await {
            let msg = match msg {
                Ok(Message::Text(t)) => t,
                Ok(Message::Close(_)) => break,
                Ok(_) => continue,
                Err(e) => {
                    warn!(error = %e, "ws receive error");
                    break;
                }
            };
            let parsed: Result<WsMessage, _> = serde_json::from_str(&msg);
            let envelope = match parsed {
                Ok(e) => e,
                Err(e) => {
                    warn!(error = %e, "ws parse error; payload={msg}");
                    let _ = out_tx.send(json!({
                        "kind": "event",
                        "event": "error",
                        "data": { "code": 1001, "message": format!("parse: {e}") }
                    }));
                    continue;
                }
            };
            match envelope {
                WsMessage::Request { id, method, params } => {
                    let result = dispatch(&method, params, Arc::clone(&state2)).await;
                    let response = match result {
                        Ok(value) => json!({
                            "kind": "response",
                            "id": id,
                            "result": value,
                            "error": null,
                        }),
                        Err((code, message)) => json!({
                            "kind": "response",
                            "id": id,
                            "result": null,
                            "error": { "code": code, "message": message, "data": null },
                        }),
                    };
                    if out_tx.send(response).is_err() {
                        break;
                    }
                }
                WsMessage::Cancel { id } => {
                    // v0.0.3: no in-flight cancellation yet
                    let _ = out_tx.send(json!({
                        "kind": "event",
                        "event": "cancelled",
                        "data": { "id": id }
                    }));
                }
                WsMessage::Response { .. } | WsMessage::Event { .. } => {
                    // Client shouldn't send these
                }
            }
        }
    });

    // Bridge: forward events broadcast to peers → out_tx.
    // We add a second peer entry whose sender is out_tx so broadcast()
    // automatically delivers events to this socket.
    state.peers.lock().await.push(PeerHandle {
        id: peer_id,
        sender: out_tx.clone(),
    });

    // Wait for either task to finish.
    tokio::select! {
        _ = write_task => {},
        _ = read_task => {},
    }

    crate::peers::remove_peer(&state.peers, peer_id).await;
    info!(%peer_id, "ws disconnected");
}

fn json_response(v: Value) -> (StatusCode, String) {
    (StatusCode::OK, v.to_string())
}

// Unused but available for future routes
#[allow(dead_code)]
async fn placeholder_path(Path(_): Path<String>) -> &'static str {
    "placeholder"
}
