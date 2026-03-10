use tokio::net::UdpSocket;
use std::time::Duration;
use anyhow::{Result, Context};

/// Determines the public IP and Port of this node by asking a STUN server.
/// This allows edge nodes (Laptops, Mobile) behind household NATs to 
/// embed their true routing address in the NodeDescriptor.
pub async fn discover_public_address(stun_server: &str) -> Result<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").await.context("Failed to bind UDP socket")?;
    let local_port = socket.local_addr()?.port();
    tracing::info!("NAT Traversal: Local UDP bound to port {}", local_port);
    tracing::info!("NAT Traversal: Sending pure STUN BINDING_REQUEST to {}", stun_server);

    // Simple raw STUN Binding Request packet (RFC 5389)
    // 0x0001 (Binding Request)
    // 0x0000 (Message Length: 0)
    // 0x2112A442 (Magic Cookie)
    // 12 bytes of random transaction ID
    let mut request = vec![
        0x00, 0x01, // Binding Request
        0x00, 0x00, // Length: 0
        0x21, 0x12, 0xA4, 0x42, // Magic Cookie
    ];
    let transaction_id: [u8; 12] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    request.extend_from_slice(&transaction_id);

    socket.send_to(&request, stun_server).await.context("Failed to send STUN request")?;

    let mut buf = [0u8; 1024];
    let (len, _) = tokio::time::timeout(Duration::from_secs(5), socket.recv_from(&mut buf))
        .await
        .context("STUN response timed out")?
        .context("Failed to receive STUN response")?;

    if len < 20 {
        return Err(anyhow::anyhow!("STUN response too short"));
    }

    // Parse the STUN response attributes to find XOR-MAPPED-ADDRESS
    let mut pos = 20;
    while pos + 4 <= len {
        let attr_type = u16::from_be_bytes([buf[pos], buf[pos + 1]]);
        let attr_len = u16::from_be_bytes([buf[pos + 2], buf[pos + 3]]) as usize;
        pos += 4;

        if pos + attr_len > len { break; }

        if attr_type == 0x0020 { // XOR-MAPPED-ADDRESS
            let family = buf[pos + 1];
            if family == 0x01 { // IPv4
                let xor_port = u16::from_be_bytes([buf[pos + 2], buf[pos + 3]]);
                let port = xor_port ^ 0x2112; // XOR with top 16 bits of magic cookie

                let ip_bytes = [
                    buf[pos + 4] ^ 0x21,
                    buf[pos + 5] ^ 0x12,
                    buf[pos + 6] ^ 0xA4,
                    buf[pos + 7] ^ 0x42,
                ];
                let mapped_address = format!("{}.{}.{}.{}:{}", ip_bytes[0], ip_bytes[1], ip_bytes[2], ip_bytes[3], port);
                
                tracing::info!("NAT Traversal Success: External IP is {}", mapped_address);
                return Ok(mapped_address);
            }
        }
        
        // STUN attributes are padded to 4-byte boundaries
        let padding = (4 - (attr_len % 4)) % 4;
        pos += attr_len + padding;
    }

    Err(anyhow::anyhow!("Failed to find XOR-MAPPED-ADDRESS in STUN response"))
}
