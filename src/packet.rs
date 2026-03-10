// src/packet.rs
// ============================================================
// SUPERNODE — Packet Protocol
// Binary wire format for all node-to-node communication.
//
// Design goals:
//   - Fixed-size 32-byte header → SIMD-parseable
//   - Variable-length payload
//   - TTL field prevents routing loops
//   - Priority field for QoS (SuperNode heartbeats > user data)
// ============================================================

use crate::node::NodeId;
use serde::{Deserialize, Serialize};

// ── Packet Types ───────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(u8)]
pub enum PacketType {
    // ── Control plane ──
    Handshake       = 0x01,  // Node join handshake
    Heartbeat       = 0x02,  // Keep-alive ping
    HeartbeatAck    = 0x03,  // Pong
    RouteUpdate     = 0x04,  // Routing table delta
    NodeJoin        = 0x05,  // New node announcement (gossip)
    NodeLeave       = 0x06,  // Node departure (gossip)
    TopologyQuery   = 0x07,  // Request neighbor list

    // ── Data plane ──
    Data            = 0x10,  // User/application data
    DataAck         = 0x11,  // Delivery acknowledgment
    DataNack        = 0x12,  // Delivery failure (triggers reroute)
    Multicast       = 0x13,  // One → many delivery
    Broadcast       = 0x14,  // Flood to all nodes (use sparingly!)

    // ── Compute plane ──
    TaskSubmit      = 0x20,  // Distribute compute task
    TaskResult      = 0x21,  // Return computation result
    TaskCancel      = 0x22,

    // ── Storage plane ──
    StoreGet        = 0x30,  // Retrieve from DHT
    StorePut        = 0x31,  // Write to DHT
    StoreDelete     = 0x32,
    StoreAck        = 0x33,
}

// ── Priority ───────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum Priority {
    Critical  = 0,   // System control — always delivered first
    High      = 1,   // Real-time (voice, video frames)
    Normal    = 2,   // Default
    Low       = 3,   // Background sync, bulk data
    Bulk      = 4,   // Transfers that fill spare capacity
}

// ── Packet Header ──────────────────────────────────────────
/// 32-byte fixed header — parsed without allocation
/// Layout (all big-endian):
///   [0..16]  source node ID
///   [16..32] destination node ID  
///   [32..40] sequence number
///   [40..48] timestamp (unix microseconds)
///   [48]     packet type
///   [49]     TTL (max hops before drop)
///   [50]     priority
///   [51]     flags (bit field)
///   [52..56] payload length
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketHeader {
    pub source:       NodeId,
    pub destination:  NodeId,
    pub seq:          u64,
    pub timestamp_us: u64,
    pub pkt_type:     PacketType,
    pub ttl:          u8,       // Starts at 64, decremented each hop
    pub priority:     Priority,
    pub flags:        PacketFlags,
    pub payload_len:  u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PacketFlags {
    pub compressed:  bool,  // Payload is lz4 compressed
    pub encrypted:   bool,  // Payload is end-to-end encrypted
    pub fragmented:  bool,  // Part of a larger fragmented message
    pub ack_request: bool,  // Sender wants delivery confirmation
    pub multipath:   bool,  // Duplicate via multiple paths (high reliability)
}

// ── Full Packet ────────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Packet {
    pub header:  PacketHeader,
    pub payload: Vec<u8>,
}

impl Packet {
    pub fn new_data(source: NodeId, dest: NodeId, data: Vec<u8>) -> Self {
        Packet {
            header: PacketHeader {
                source,
                destination: dest,
                seq: 0,
                timestamp_us: current_time_us(),
                pkt_type: PacketType::Data,
                ttl: 64,
                priority: Priority::Normal,
                flags: PacketFlags::default(),
                payload_len: data.len() as u32,
            },
            payload: data,
        }
    }

    pub fn new_heartbeat(source: NodeId, dest: NodeId) -> Self {
        Packet {
            header: PacketHeader {
                source,
                destination: dest,
                seq: 0,
                timestamp_us: current_time_us(),
                pkt_type: PacketType::Heartbeat,
                ttl: 1,  // Heartbeats don't traverse multiple hops
                priority: Priority::Critical,
                flags: PacketFlags::default(),
                payload_len: 0,
            },
            payload: vec![],
        }
    }

    pub fn new_broadcast(source: NodeId, data: Vec<u8>) -> Self {
        // Broadcast destination = all-zeros ID by convention
        let dest = NodeId(0);
        Packet {
            header: PacketHeader {
                source,
                destination: dest,
                seq: 0,
                timestamp_us: current_time_us(),
                pkt_type: PacketType::Broadcast,
                ttl: 7,   // Limit broadcast radius
                priority: Priority::Low,
                flags: PacketFlags { compressed: true, ..Default::default() },
                payload_len: data.len() as u32,
            },
            payload: data,
        }
    }

    /// Decrement TTL — returns false if packet should be dropped
    pub fn decrement_ttl(&mut self) -> bool {
        if self.header.ttl == 0 {
            return false;
        }
        self.header.ttl -= 1;
        self.header.ttl > 0
    }

    pub fn total_size(&self) -> usize {
        std::mem::size_of::<PacketHeader>() + self.payload.len()
    }
}

fn current_time_us() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros() as u64
}

// ── Message (High-level Application Layer) ─────────────────
/// Application-level messages that sit on top of packets.
/// A single message may be fragmented across multiple packets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id:       u64,           // Unique message ID
    pub from:     NodeId,
    pub to:       NodeId,        // Single dest, group ID, or 0 for broadcast
    pub topic:    String,        // Pub/sub topic
    pub payload:  Vec<u8>,       // Raw bytes (could be JSON, protobuf, etc.)
    pub reply_to: Option<u64>,   // For request/response patterns
    pub ttl_ms:   u32,           // Message expiry
}
