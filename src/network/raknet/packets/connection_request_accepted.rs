use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};

use bytes::{BufMut, BytesMut};

use crate::instance::IPV4_LOCAL_ADDR;
use crate::network::traits::Encodable;
use crate::util::{EMPTY_IPV4_ADDRESS, IPV4_MEM_SIZE, IPV6_MEM_SIZE, WriteExtensions};

/// Sent in response to [`ConnectionRequest`](super::connection_request::ConnectionRequest).
#[derive(Debug)]
pub struct ConnectionRequestAccepted {
    /// IP address of the client.
    pub client_address: SocketAddr,
    /// Corresponds to [`ConnectionRequest::time`](super::connection_request::ConnectionRequest::time).
    pub request_time: i64,
}

impl ConnectionRequestAccepted {
    /// Unique ID of this packet.
    pub const ID: u8 = 0x10;
}

impl Encodable for ConnectionRequestAccepted {
    fn encode(&self) -> anyhow::Result<BytesMut> {
        let mut buffer =
            BytesMut::with_capacity(1 + IPV6_MEM_SIZE + 2 + 10 * IPV4_MEM_SIZE + 8 + 8);

        buffer.put_u8(Self::ID);
        buffer.put_addr(self.client_address);
        buffer.put_i16(0); // System index
        for _ in 0..20 {
            // 20 internal IDs
            buffer.put_addr(*EMPTY_IPV4_ADDRESS);
        }
        buffer.put_i64(self.request_time);
        buffer.put_i64(self.request_time); // Response time

        Ok(buffer)
    }
}