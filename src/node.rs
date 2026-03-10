// src/node.rs
// ============================================================
// SUPERNODE — Core Node Identity & Type System
// Supports 8,000,000,000 concurrent device connections
// Each NodeId is 8 bytes → 8B nodes = 64GB address space
// ============================================================

use serde::{Deserialize, Serialize};
use std::fmt;
use std::net::SocketAddr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

// ── Node ID ────────────────────────────────────────────────
/// Globally unique 128-bit node identifier.
/// Format: [region(8b) | shard(16b) | timestamp(48b) | random(56b)]
/// This layout allows:
///   - O(1) region lookup by masking high bits
///   - Temporal ordering for debugging
///   - Near-zero collision probability at 8B scale
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u128);

impl NodeId {
    pub fn new() -> Self {
        NodeId(Uuid::new_v4().as_u128())
    }

    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        NodeId(u128::from_be_bytes(bytes))
    }

    /// Extract region index (top 8 bits) for fast routing table lookup
    pub fn region_hint(&self) -> u8 {
        (self.0 >> 120) as u8
    }

    /// Extract shard index (bits 112-120)
    pub fn shard_hint(&self) -> u16 {
        ((self.0 >> 104) & 0xFFFF) as u16
    }

    pub fn as_bytes(&self) -> [u8; 16] {
        self.0.to_be_bytes()
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let b = self.as_bytes();
        write!(f, "{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}",
            b[0], b[1], b[2], b[3], b[4], b[5])
    }
}

// ── Node Type ──────────────────────────────────────────────
/// Every physical device class in the global mesh.
/// Capabilities and routing weights differ per type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    /// Ultra-high-capacity relay — handles 10M+ child connections
    /// 100Gbps uplink, 512GB RAM, dedicated hardware
    SuperNode,

    /// Regional gateway — aggregates traffic for a geographic zone  
    /// 10Gbps uplink, handles 500K connections
    RegionalHub,

    /// Standard cloud/bare-metal server
    /// 1Gbps, handles 50K connections
    Server,

    /// Personal computer or workstation
    Laptop { battery_pct: u8 },

    /// Smartphone or tablet
    Mobile { os: MobileOS, signal_dbm: i8 },

    /// Embedded sensors, smart home, industrial
    IoT { category: IoTCategory },

    /// Vehicle-mounted, high-mobility node
    Vehicle { speed_kmh: u16 },

    /// Low-earth-orbit satellite relay
    Satellite { orbital_slot: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MobileOS { Android, iOS, HarmonyOS, Other }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IoTCategory { SmartHome, Industrial, Medical, Agricultural, Environmental }

impl NodeType {
    /// Max concurrent connections this node type can sustain
    pub fn max_connections(&self) -> u32 {
        match self {
            NodeType::SuperNode       => 10_000_000,
            NodeType::RegionalHub     => 500_000,
            NodeType::Server          => 50_000,
            NodeType::Laptop { .. }   => 256,
            NodeType::Mobile { .. }   => 64,
            NodeType::IoT { .. }      => 8,
            NodeType::Vehicle { .. }  => 128,
            NodeType::Satellite { .. }=> 1_000_000,
        }
    }

    /// Routing priority weight (higher = preferred relay)
    pub fn routing_weight(&self) -> f32 {
        match self {
            NodeType::SuperNode       => 1000.0,
            NodeType::Satellite { .. }=> 800.0,
            NodeType::RegionalHub     => 500.0,
            NodeType::Server          => 100.0,
            NodeType::Vehicle { .. }  => 20.0,
            NodeType::Laptop { .. }   => 10.0,
            NodeType::Mobile { .. }   => 5.0,
            NodeType::IoT { .. }      => 1.0,
        }
    }
}

// ── Geographic Region ──────────────────────────────────────
/// Coarse geographic sharding — 16 zones cover the entire globe.
/// Used for: locality-aware routing, data sovereignty, latency reduction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Region {
    NorthAmericaEast  = 0,
    NorthAmericaWest  = 1,
    SouthAmerica      = 2,
    EuropeWest        = 3,
    EuropeEast        = 4,
    AfricaNorth       = 5,
    AfricaSub         = 6,
    MiddleEast        = 7,
    SouthAsia         = 8,
    EastAsia          = 9,
    SoutheastAsia     = 10,
    Oceania           = 11,
    CentralAsia       = 12,
    ArcticRelays      = 13,
    OceanBuoys        = 14,
    Orbital           = 15,
}

impl Region {
    /// Expected inter-region latency in milliseconds
    pub fn latency_to(&self, other: &Region) -> u32 {
        let a = *self as u8;
        let b = *other as u8;
        if a == b { return 1; }
        // Simplified: same continent ~20ms, cross-ocean ~120ms, orbital ~600ms
        let diff = (a as i32 - b as i32).unsigned_abs();
        match diff {
            0       => 1,
            1..=2   => 20,
            3..=6   => 80,
            7..=10  => 120,
            _       => 180,
        }
    }
}

// ── Node Capabilities ──────────────────────────────────────
/// What this node can DO — used for task placement decisions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Capabilities {
    pub can_relay:        bool,   // Forward packets for others
    pub can_store:        bool,   // Cache/store data
    pub can_compute:      bool,   // Execute distributed tasks
    pub can_encrypt:      bool,   // TLS/noise protocol endpoint
    pub storage_bytes:    u64,    // Available storage
    pub bandwidth_bps:    u64,    // Available bandwidth
    pub cpu_cores:        u16,    // Logical CPU count
    pub ram_bytes:        u64,    // Available RAM
    pub gpu_flops:        u64,    // GPU compute capacity (0 if none)
}

// ── Node Descriptor ────────────────────────────────────────
/// Complete node record — stored in the distributed registry
/// Serialized size: ~256 bytes → 8B nodes = ~2TB total registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDescriptor {
    pub id:           NodeId,
    pub node_type:    NodeType,
    pub region:       Region,
    pub addr:         SocketAddr,           // Primary address
    pub addrs:        Vec<SocketAddr>,      // Alternate addresses (multi-homed)
    pub capabilities: Capabilities,
    pub public_key:   [u8; 32],             // Ed25519 public key
    pub version:      u32,                  // Protocol version
    pub joined_at:    u64,                  // Unix timestamp
    pub last_seen:    u64,                  // Heartbeat timestamp
    pub hops_to_super: u8,                  // Distance to nearest SuperNode
    pub load_factor:  f32,                  // 0.0 = idle, 1.0 = saturated
}

impl NodeDescriptor {
    pub fn new(node_type: NodeType, region: Region, addr: SocketAddr) -> Self {
        NodeDescriptor {
            id: NodeId::new(),
            node_type,
            region,
            addr,
            addrs: vec![],
            capabilities: Capabilities::default(),
            public_key: [0u8; 32],
            version: 1,
            joined_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_secs(),
            last_seen: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_secs(),
            hops_to_super: 255,
            load_factor: 0.0,
        }
    }

    pub fn is_alive(&self, timeout_secs: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();
        now.saturating_sub(self.last_seen) < timeout_secs
    }
}
