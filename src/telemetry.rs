use metrics::{describe_counter, describe_histogram, describe_gauge};
use metrics_exporter_prometheus::PrometheusBuilder;
use std::net::SocketAddr;

pub fn init_telemetry(bind_addr: SocketAddr) {
    // Build Prometheus exporter
    let builder = PrometheusBuilder::new();
    
    // Attempt to install the exporter, which binds a background HTTP server
    match builder.with_http_listener(bind_addr).install() {
        Ok(_) => {
            tracing::info!("Metrics exporter successfully running on http://{}/metrics", bind_addr);
        }
        Err(e) => {
            tracing::error!("Failed to install Prometheus metrics exporter: {}", e);
        }
    }

    // Register primary metrics with descriptions
    describe_counter!("supernode.packets_routed", "Total number of network packets successfully routed");
    describe_counter!("supernode.packets_dropped", "Total number of network packets dropped (e.g. TTL expired)");
    describe_gauge!("supernode.active_connections", "Current number of active TCP/QUIC connections");
    describe_histogram!("supernode.routing_latency_us", "Microseconds spent computing routes via Dijkstra/A*");
}
