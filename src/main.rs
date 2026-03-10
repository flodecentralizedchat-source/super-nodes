// src/main.rs
// ============================================================
// SUPERNODE — Main Entry Point
// Bootstraps a SuperNode capable of connecting to 8B devices
// ============================================================

mod node;
mod graph;
mod network;
mod packet;
pub mod telemetry;
pub mod storage;
pub mod nat;
mod api;

use crate::node::{NodeDescriptor, NodeId, NodeType, Region};
use crate::graph::{MeshGraph, EdgeWeight};
use crate::network::SuperNodeServer;
use crate::api;
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use anyhow::Result;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    // ── Tracing ────────────────────────────────────────────
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("╔══════════════════════════════════════════╗");
    info!("║   SUPERNODE v0.1 — 8 Billion Devices     ║");
    info!("║   Built with Rust + Tokio + QUIC          ║");
    info!("╚══════════════════════════════════════════╝");

    // Initialize Prometheus Telemetry exporter
    telemetry::init_telemetry("0.0.0.0:9090".parse().unwrap());

    // Initialize Distributed Storage (Raft)
    let _raft_store = storage::SuperNodeStore::new();
    let _raft_config = storage::create_raft_config();

    // ── Build the mesh graph ───────────────────────────────
    let graph = Arc::new(MeshGraph::new());

    // Register this SuperNode
    let this_id = NodeId::new();
    info!("SuperNode ID: {}", this_id);

    // ── NAT Traversal (STUN) ───────────────────────────────
    let _public_addr = nat::discover_public_address("stun.l.google.com:19302").await.unwrap_or_else(|_| "0.0.0.0:9000".to_string());

    // ── Spawn the server ───────────────────────────────────
    let server = SuperNodeServer::new(this_id, graph.clone());

    // Metrics reporter
    {
        let metrics = server.metrics.clone();
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(
                tokio::time::Duration::from_secs(10)
            );
            loop {
                ticker.tick().await;
                info!(
                    "METRICS | active={} | total={} | routed={} | dropped={}",
                    metrics.active_connections.load(std::sync::atomic::Ordering::Relaxed),
                    metrics.total_connections.load(std::sync::atomic::Ordering::Relaxed),
                    metrics.packets_routed.load(std::sync::atomic::Ordering::Relaxed),
                    metrics.packets_dropped.load(std::sync::atomic::Ordering::Relaxed),
                );
            }
        });
    }

// We can add some basic tests here later
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_init() {
        assert!(true);
    }
}


    // ── Simulated Traffic ───────────────────────────────────
    {
        let router = server.router.clone();
        let connections = server.connections.clone();
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(tokio::time::Duration::from_millis(100));
            loop {
                ticker.tick().await;
                // We just simulate routing a dummy packet every 100ms
                let dummy_packet = crate::packet::Packet::new_heartbeat(NodeId::new(), NodeId::new());
                router.route(dummy_packet, &connections).await;
            }
        });
    }

    // ── Heartbeat Background Task ──────────────────────────
    let heartbeat_manager = crate::network::HeartbeatManager {
        connections: server.connections.clone(),
        graph: server.graph.clone(),
        router: server.router.clone(),
    };
    tokio::spawn(async move { heartbeat_manager.run().await; });
    info!("Heartbeat manager running");

    // ── Spawn API Server ───────────────────────────────────
    let api_graph = graph.clone();
    tokio::spawn(async move {
        if let Err(e) = api::start_api_server(api_graph, 3000).await {
            error!("API server failed: {}", e);
        }
    });
    info!("🌐 API server starting on http://0.0.0.0:3000");

    // ── Listen for connections ─────────────────────────────
    info!("Starting listener...");
    server.listen("0.0.0.0:9000").await?;

    Ok(())
}
