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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
    RaftPayload     = 0x34,  // OpenRaft consensus log
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
    #[allow(dead_code)]
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

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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

    #[allow(dead_code)]
  pub fn new_ack(source: NodeId, dest: NodeId, acked_seq: u64) -> Self {
        Packet {
            header: PacketHeader {
                source,
                destination: dest,
                seq: 0,
               timestamp_us: current_time_us(),
                pkt_type: PacketType::DataAck,
                ttl: 64,
                priority: Priority::High,
                flags: PacketFlags::default(),
                payload_len: 0,
            },
            payload: acked_seq.to_be_bytes().to_vec(),
        }
    }

    #[allow(dead_code)]
  pub fn new_nack(source: NodeId, dest: NodeId, failed_seq: u64, reason: &str) -> Self {
       let reason_bytes = reason.as_bytes();
       let mut payload = failed_seq.to_be_bytes().to_vec();
        payload.extend_from_slice(reason_bytes);
        
        Packet {
            header: PacketHeader {
                source,
                destination: dest,
                seq: 0,
               timestamp_us: current_time_us(),
                pkt_type: PacketType::DataNack,
                ttl: 64,
                priority: Priority::High,
                flags: PacketFlags::default(),
                payload_len: payload.len() as u32,
            },
            payload,
        }
    }

    /// Validate that header payload_len matches actual payload size
    #[allow(dead_code)]
  pub fn validate(&self) -> bool {
        self.header.payload_len == self.payload.len() as u32
    }

    /// Set compression flag
    #[allow(dead_code)]
  pub fn set_compressed(&mut self, compressed: bool) {
        self.header.flags.compressed = compressed;
    }

    /// Set encryption flag
    #[allow(dead_code)]
  pub fn set_encrypted(&mut self, encrypted: bool) {
        self.header.flags.encrypted = encrypted;
    }

    /// Request acknowledgment
    #[allow(dead_code)]
  pub fn request_ack(&mut self) {
        self.header.flags.ack_request = true;
    }

    /// Decrement TTL — returns false if packet should be dropped
    #[allow(dead_code)]
  pub fn decrement_ttl(&mut self) -> bool {
        if self.header.ttl == 0 {
            return false;
        }
        self.header.ttl -= 1;
        self.header.ttl > 0
    }

    #[allow(dead_code)]
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
#[allow(dead_code)]
pub struct Message {
   pub id:      u64,           // Unique message ID
   pub from:     NodeId,
   pub to:       NodeId,        // Single dest, group ID, or 0 for broadcast
   pub topic:    String,        // Pub/sub topic
   pub payload:  Vec<u8>,       // Raw bytes(could be JSON, protobuf, etc.)
   pub reply_to: Option<u64>,   // For request/response patterns
   pub ttl_ms:  u32,           // Message expiry
}

// ── Tests ──────────────────────────────────────────────────
#[cfg(test)]
mod tests {
  use super::*;

    #[test]
    fn test_packet_type_discriminants() {
        assert_eq!(PacketType::Handshake as u8, 0x01);
        assert_eq!(PacketType::Heartbeat as u8, 0x02);
        assert_eq!(PacketType::Data as u8, 0x10);
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Critical > Priority::High);
        assert!(Priority::Normal > Priority::Low);
    }

    #[test]
    fn test_new_data_packet() {
      let src = NodeId::new();
      let dst = NodeId::new();
      let data = vec![1, 2, 3, 4, 5];
        
      let packet = Packet::new_data(src, dst, data.clone());
        
        assert_eq!(packet.header.source, src);
        assert_eq!(packet.header.pkt_type, PacketType::Data);
        assert_eq!(packet.header.ttl, 64);
        assert_eq!(packet.payload, data);
        assert!(packet.validate());
    }

    #[test]
    fn test_new_ack_packet() {
      let src = NodeId::new();
      let dst = NodeId::new();
      let acked_seq = 12345u64;
        
      let packet = Packet::new_ack(src, dst, acked_seq);
        
        assert_eq!(packet.header.pkt_type, PacketType::DataAck);
        assert_eq!(packet.header.priority, Priority::High);
      let restored_seq = u64::from_be_bytes(packet.payload.try_into().unwrap());
        assert_eq!(restored_seq, acked_seq);
    }

    #[test]
    fn test_ttl_decrement() {
      let src = NodeId::new();
      let dst = NodeId::new();
      let mut packet = Packet::new_data(src, dst, vec![1]);
        
       packet.header.ttl = 5;
        assert!(packet.decrement_ttl());
        assert_eq!(packet.header.ttl, 4);
        
       packet.header.ttl = 1;
        assert!(!packet.decrement_ttl());
        assert_eq!(packet.header.ttl, 0);
    }

    #[test]
    fn test_packet_flags() {
      let src = NodeId::new();
      let dst = NodeId::new();
      let mut packet = Packet::new_data(src, dst, vec![1]);
        
        assert!(!packet.header.flags.compressed);
       packet.set_compressed(true);
        assert!(packet.header.flags.compressed);
        
       packet.request_ack();
        assert!(packet.header.flags.ack_request);
    }

    #[test]
    fn test_packet_validation() {
      let src = NodeId::new();
      let dst = NodeId::new();
      let mut packet = Packet::new_data(src, dst, vec![1, 2, 3]);
        
        assert!(packet.validate());
        
       packet.header.payload_len = 100;
        assert!(!packet.validate());
    }

    #[test]
    fn test_packet_size_calculation() {
      let src = NodeId::new();
      let dst = NodeId::new();
      let data = vec![1, 2, 3, 4, 5];
        
      let packet = Packet::new_data(src, dst, data);
      let expected_size = std::mem::size_of::<PacketHeader>() + 5;
        
        assert_eq!(packet.total_size(), expected_size);
    }
}
