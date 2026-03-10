use openraft::{Raft, Config};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

// The application data stored in Raft
// We are storing the global topology state (e.g. active supernodes)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TopologyRequest {
    AddSuperNode { id: u64, public_ip: String },
    RemoveSuperNode { id: u64 },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TopologyResponse {
    pub success: bool,
}

// In a full implementation, we would implement `openraft::RaftLogReader`,
// `RaftStorage`, and `RaftNetwork` traits over our MeshGraph.
// For this architecture phase, we define the foundational structures.
pub struct SuperNodeStore {
    // In-memory state machine
    pub state: RwLock<std::collections::HashMap<u64, String>>,
}

impl SuperNodeStore {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            state: RwLock::new(std::collections::HashMap::new()),
        })
    }
}

// Basic init config for the Raft node
pub fn create_raft_config() -> Arc<Config> {
    Arc::new(Config {
        heartbeat_interval: 250,
        election_timeout_min: 500,
        election_timeout_max: 1000,
        ..Default::default()
    })
}
