// src/api.rs
// ============================================================
// SUPERNODE — HTTP/WebSocket API Server
// Provides REST API and real-time WebSocket for algorithm streaming
// ============================================================

use axum::{
    extract::{Path, State, ws::{WebSocket, Message, WebSocketUpgrade}},
   http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::Duration,
};
use tokio::sync::broadcast;
use tracing::{info, warn, error};

use crate::{
    graph::MeshGraph,
    node::{NodeDescriptor, NodeId, NodeType},
    telemetry,
};

// ============================================================
// API State
// ============================================================

#[derive(Clone)]
pub struct ApiState {
    pub graph: Arc<MeshGraph>,
    pub algorithm_tx: broadcast::Sender<AlgorithmEvent>,
}

// ============================================================
// Algorithm Events (streamed via WebSocket)
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AlgorithmEvent {
    #[serde(rename = "algorithm_started")]
    Started { 
        algorithm: String,
        timestamp: u64,
    },
    
    #[serde(rename = "path_found")]
    PathFound {
        path: Vec<String>,
        total_distance: f64,
        computation_time_ms: u64,
    },
    
    #[serde(rename = "gossip_step")]
    GossipStep {
        step: u32,
        infected_nodes: Vec<String>,
        message_count: u32,
    },
    
    #[serde(rename = "mst_edge")]
    MstEdge {
        from: String,
        to: String,
        weight: f64,
        total_weight: f64,
    },
    
    #[serde(rename = "bfs_visit")]
    BfsVisit {
        node: String,
        distance: u32,
        visited_count: u32,
    },
    
    #[serde(rename = "algorithm_complete")]
    Complete {
        success: bool,
        result_summary: String,
        total_time_ms: u64,
    },
    
    #[serde(rename = "error")]
    Error {
        message: String,
    },
}

// ============================================================
// Response Types
// ============================================================

#[derive(Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub uptime_seconds: u64,
    pub node_count: usize,
    pub active_connections: u32,
}

#[derive(Serialize, Deserialize)]
pub struct RunAlgorithmRequest {
    pub algorithm: String,
    #[serde(default)]
    pub params: HashMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct RunAlgorithmResponse {
    pub success: bool,
    pub event_id: String,
    pub message: String,
}

// ============================================================
// Router Setup
// ============================================================

pub fn create_router(state: ApiState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/api/algorithms", get(list_algorithms))
        .route("/api/algorithms/run", post(run_algorithm))
        .route("/api/algorithms/:algo_id/status", get(algorithm_status))
        .route("/ws/algorithms", websocket_handler)
        .route("/metrics", get(metrics_endpoint))
        .with_state(state)
}

// ============================================================
// HTTP Handlers
// ============================================================

async fn health_check(State(state): State<ApiState>) -> Json<HealthResponse> {
    let node_count = state.graph.node_count();
    
    Json(HealthResponse {
        status: "healthy".to_string(),
        uptime_seconds: 0, // TODO: Track actual uptime
        node_count,
        active_connections: 0,
    })
}

async fn list_algorithms() -> Json<Vec<AlgorithmInfo>> {
    Json(vec![
        AlgorithmInfo {
            id: "dijkstra".to_string(),
            name: "Dijkstra's Shortest Path".to_string(),
            description: "Find optimal path between two nodes".to_string(),
            complexity: "O((V+E) log V)".to_string(),
        },
        AlgorithmInfo {
            id: "astar".to_string(),
            name: "A* Pathfinding".to_string(),
            description: "Heuristic-guided pathfinding with Manhattan distance".to_string(),
            complexity: "O(b^d)".to_string(),
        },
        AlgorithmInfo {
            id: "gossip".to_string(),
            name: "Gossip Protocol".to_string(),
            description: "Epidemic message propagation simulation".to_string(),
            complexity: "O(n log n)".to_string(),
        },
        AlgorithmInfo {
            id: "kruskal".to_string(),
            name: "Kruskal's MST".to_string(),
            description: "Minimum Spanning Tree using union-find".to_string(),
            complexity: "O(E log E)".to_string(),
        },
        AlgorithmInfo {
            id: "bfs".to_string(),
            name: "Breadth-First Search".to_string(),
            description: "Level-order graph traversal".to_string(),
            complexity: "O(V+E)".to_string(),
        },
    ])
}

async fn run_algorithm(
    State(state): State<ApiState>,
    Json(payload): Json<RunAlgorithmRequest>,
) -> Result<Json<RunAlgorithmResponse>, StatusCode> {
    info!("Running algorithm: {}", payload.algorithm);
    
    let event_id = uuid::Uuid::new_v4().to_string();
    
    // Broadcast algorithm start event
    let start_event = AlgorithmEvent::Started {
        algorithm: payload.algorithm.clone(),
        timestamp: chrono::Utc::now().timestamp_millis() as u64,
    };
    
    if let Err(e) = state.algorithm_tx.send(start_event) {
        error!("Failed to broadcast algorithm start: {}", e);
    }
    
    // TODO: Actually run the algorithm based on payload.algorithm
    // For now, just acknowledge the request
    
    Ok(Json(RunAlgorithmResponse {
        success: true,
        event_id,
        message: format!("Algorithm {} started", payload.algorithm),
    }))
}

async fn algorithm_status(
    Path(algo_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // TODO: Implement actual status tracking
    Ok(Json(serde_json::json!({
        "algorithm": algo_id,
        "status": "idle",
        "progress": 0
    })))
}

async fn metrics_endpoint() -> String {
    // Export Prometheus metrics
    telemetry::export_metrics()
}

// ============================================================
// WebSocket Handler
// ============================================================

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<ApiState>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |socket| handle_websocket(socket, state))
}

async fn handle_websocket(socket: WebSocket, state: ApiState) {
    let (mut sender, mut receiver) = socket.split();
    
    // Subscribe to algorithm events
    let mut event_rx = state.algorithm_tx.subscribe();
    
    // Spawn task to receive messages from client
    let recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    info!("Received from client: {}", text);
                    // Handle client commands if needed
                }
                Ok(Message::Close(_)) => {
                    info!("Client disconnected");
                    break;
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });
    
    // Spawn task to send algorithm events to client
    let send_task = tokio::spawn(async move {
        while let Ok(event) = event_rx.recv().await {
            match serde_json::to_string(&event) {
                Ok(json) => {
                    if let Err(e) = sender.send(Message::Text(json)).await {
                        error!("Failed to send event: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    error!("Failed to serialize event: {}", e);
                }
            }
        }
    });
    
    // Wait for either task to complete
    tokio::select! {
        _ = recv_task => {
            info!("Receive task completed");
        }
        _ = send_task => {
            info!("Send task completed");
        }
    }
}

// ============================================================
// Helper Types
// ============================================================

#[derive(Serialize, Deserialize)]
struct AlgorithmInfo {
    id: String,
    name: String,
    description: String,
    complexity: String,
}

// ============================================================
// Server Initialization
// ============================================================

pub async fn start_api_server(
    graph: Arc<MeshGraph>,
    port: u16,
) -> anyhow::Result<()> {
    let (algorithm_tx, _) = broadcast::channel::<AlgorithmEvent>(100);
    
    let state = ApiState {
        graph,
        algorithm_tx,
    };
    
    let app = create_router(state);
    
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    info!("🚀 Starting API server on {}", addr);
    
    let listener= tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
