use metrics::{describe_counter, describe_histogram, describe_gauge, counter, histogram, gauge};
use metrics_exporter_prometheus::PrometheusBuilder;
use std::net::SocketAddr;
use std::time::Instant;

/// Initialize telemetry system with Prometheus metrics exporter.
/// Sets up HTTP server on bind_addr serving /metrics endpoint.
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
    
    // Register additional metrics
   describe_counter!("supernode.bytes_sent", "Total bytes transmitted over network");
   describe_counter!("supernode.bytes_received", "Total bytes received from network");
   describe_counter!("supernode.connections_total", "Total number of connections accepted since startup");
   describe_counter!("supernode.handshakes_failed", "Number of failed QUIC/TLS handshakes");
   describe_counter!("supernode.nodes_added", "Number of nodes added to mesh graph");
   describe_counter!("supernode.nodes_removed", "Number of nodes removed from mesh graph");
   describe_counter!("supernode.cache_hits", "Router cache hit count");
   describe_counter!("supernode.cache_misses", "Router cache miss count");
   describe_counter!("supernode.raft_commits", "Number of Raft consensus commits");
   describe_counter!("supernode.snapshots_created", "Number of Raft snapshots created");
   
   describe_gauge!("supernode.graph_nodes", "Current number of nodes in mesh graph");
   describe_gauge!("supernode.graph_edges", "Current number of edges in mesh graph");
   describe_gauge!("supernode.memory_used_bytes", "Total memory consumed by SuperNode process");
   describe_gauge!("supernode.bandwidth_bps", "Current network bandwidth in bytes per second");
   describe_gauge!("supernode.raft_term", "Current Raft consensus term");
   
   describe_histogram!("supernode.handshake_duration_ms", "Milliseconds to complete QUIC handshake");
   describe_histogram!("supernode.dht_lookup_latency_us", "Microseconds for DHT key lookup");
   describe_histogram!("supernode.snapshot_size_bytes", "Size of Raft snapshots in bytes");
}

/// Record packet routing event
pub fn record_packet_routed() {
    counter!("supernode.packets_routed").increment(1);
}

/// Record packet drop event
pub fn record_packet_dropped() {
    counter!("supernode.packets_dropped").increment(1);
}

/// Record bytes sent
pub fn record_bytes_sent(bytes: u64) {
    counter!("supernode.bytes_sent").increment(bytes);
}

/// Record bytes received
pub fn record_bytes_received(bytes: u64) {
    counter!("supernode.bytes_received").increment(bytes);
}

/// Record routing latency
pub fn record_routing_latency_us(latency_us: u64) {
    histogram!("supernode.routing_latency_us").record(latency_us as f64);
}

/// Record active connections
pub fn set_active_connections(count: u64) {
    gauge!("supernode.active_connections").set(count as f64);
}

/// Record node addition to graph
pub fn record_node_added() {
    counter!("supernode.nodes_added").increment(1);
}

/// Record node removal from graph
pub fn record_node_removed() {
    counter!("supernode.nodes_removed").increment(1);
}

/// Record cache hit
pub fn record_cache_hit() {
    counter!("supernode.cache_hits").increment(1);
}

/// Record cache miss
pub fn record_cache_miss() {
    counter!("supernode.cache_misses").increment(1);
}

/// RAII timer for measuring operation duration
pub struct OperationTimer {
    start: Instant,
    metric_name: &'static str,
}

impl OperationTimer {
  pub fn new(metric_name: &'static str) -> Self {
      Self {
            start: Instant::now(),
            metric_name,
        }
    }
}

impl Drop for OperationTimer {
   fn drop(&mut self) {
   let duration_us = self.start.elapsed().as_micros() as u64;
       histogram!(self.metric_name).record(duration_us as f64);
    }
}
