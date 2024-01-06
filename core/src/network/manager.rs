use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, AtomicU32};
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant};

use anyhow::Context;
use dashmap::DashMap;
use parking_lot::{RwLock, Mutex};
use raknet::{RaknetUser, BroadcastPacket, UserCreateInfo};
use tokio::net::UdpSocket;
use tokio::sync::{broadcast, mpsc, OnceCell};
use proto::bedrock::{ConnectedPacket, Disconnect, DISCONNECTED_TIMEOUT};
use replicator::Replicator;

use tokio_util::sync::CancellationToken;
use util::{Serialize};
use util::MutableBuffer;

use crate::config::SERVER_CONFIG;
use crate::level::LevelManager;

use super::ForwardablePacket;

const BROADCAST_CHANNEL_CAPACITY: usize = 16;
const FORWARD_TIMEOUT: Duration = Duration::from_millis(10);
const GARBAGE_COLLECT_INTERVAL: Duration = Duration::from_secs(1);

pub struct ChannelUser<T> {
    channel: mpsc::Sender<MutableBuffer>,
    state: Arc<T>
}

impl<T> ChannelUser<T> {
    #[inline]
    pub async fn forward(&self, packet: MutableBuffer) -> anyhow::Result<()> {
        self.channel.send_timeout(packet, FORWARD_TIMEOUT).await.context("Server-side client timed out")?;
        Ok(())
    }
}

pub struct ConnectedUser {
    raknet: RaknetUser
}

pub struct UserMap {
    replicator: Replicator,
    connecting_map: DashMap<SocketAddr, ChannelUser<RaknetUser>>,
    connected_map: DashMap<SocketAddr, ChannelUser<ConnectedUser>>,
    /// Channel that sends a packet to all connected sessions.
    broadcast: broadcast::Sender<BroadcastPacket>
}

impl UserMap {
    pub fn new(replicator: Replicator) -> Self {
        let connecting_map = DashMap::new();
        let connected_map = DashMap::new();

        let (broadcast, _) = broadcast::channel(BROADCAST_CHANNEL_CAPACITY);

        Self {
            replicator, connecting_map, connected_map, broadcast
        }
    }

    pub fn insert(&self, info: UserCreateInfo) {
        let (tx, rx) = mpsc::channel(BROADCAST_CHANNEL_CAPACITY);
        let state = RaknetUser::new(info, rx);

        self.connecting_map.insert(info.address, ChannelUser {
            channel: tx, state
        });
    }

    pub fn forward(&self, packet: ForwardablePacket) -> anyhow::Result<()> {
        todo!()
    }

    /// Sends a [`Disconnect`] packet to every connected user.
    /// 
    /// This does not wait for the users to actually be disconnected.
    ///
    /// # Errors
    /// 
    /// This function returns an error when the [`Disconnect`] packet fails to serialize.
    pub fn kick_all(&self, message: &str) -> anyhow::Result<()> {
        // Ignore result because it can only fail if there are no receivers remaining.
        // In that case this shouldn't do anything anyways.
        let _ = self.broadcast.send(BroadcastPacket::new(
            Disconnect {
                hide_message: false,
                message,
            },
            None,
        )?);

        Ok(())
    }

    pub fn count(&self) -> usize {
        self.connected_map.len()
    }

    pub fn max_count(&self) -> usize {
        SERVER_CONFIG.read().max_players
    }
}

//     /// Creates a new session and adds it to the tracker.
//     pub fn add_session(
//         self: &Arc<Self>,
//         ipv4_socket: Arc<UdpSocket>,
//         address: SocketAddr,
//         mtu: u16,
//         client_guid: u64,
//     ) {
//         let (sender, receiver) = mpsc::channel(BROADCAST_CHANNEL_CAPACITY);

//         let level_manager =
//             self.level_manager.get().unwrap().upgrade().unwrap();

//         let replicator = self.replicator.clone();
//         let session = Session::new(
//             self.broadcast.clone(),
//             receiver,
//             level_manager,
//             replicator,
//             ipv4_socket,
//             address,
//             mtu,
//             client_guid,
//         );

//         // Lightweight task that removes the session from the list when it is no longer active.
//         // This prevents cyclic references.
//         {
//             let list = self.list.clone();
//             let session = session.clone();

//             tokio::spawn(async move {
//                 session.cancelled().await;
//                 list.remove(&session.raknet.address);
//             });
//         }

//         self.list.insert(address, (sender, session));
//     }

//     #[inline]
//     pub fn set_level_manager(
//         &self,
//         level_manager: Weak<LevelManager>,
//     ) -> anyhow::Result<()> {
//         self.level_manager.set(level_manager)?;
//         Ok(())
//     }

//     /// Forwards a packet from the network service to the correct session.
//     pub fn forward_packet(&self, packet: RawPacket) {
//         // Spawn a new task so that the UDP receiver isn't interrupted
//         // if forwarding takes a long amount time.

//         let list = self.list.clone();
//         tokio::spawn(async move {
//             if let Some(session) = list.get(&packet.addr) {
//                 match session.0.send_timeout(packet.buf, FORWARD_TIMEOUT).await {
//                     Ok(_) => (),
//                     Err(_) => {
//                         // Session incoming queue is full.
//                         // If after a 20 ms timeout it is still full, destroy the session,
//                         // it probably froze.
//                         tracing::error!(
//                             "Closing hanging session"
//                         );

//                         // Attempt to send a disconnect packet.
//                         let _ = session.1.kick(DISCONNECTED_TIMEOUT);
//                         // Then close the session.
//                         session.1.on_disconnect();
//                     }
//                 }
//             }
//         });
//     }

//     /// Sends the given packet to every session.
//     pub fn broadcast<P: ConnectedPacket + Serialize + Clone>(
//         &self,
//         packet: P,
//     ) -> anyhow::Result<()> {
//         self.broadcast.send(BroadcastPacket::new(packet, None)?)?;
//         Ok(())
//     }

//     /// Kicks all sessions from the server, displaying the given message.
//     /// This function also waits for all sessions to be destroyed.
//     pub async fn kick_all<S: AsRef<str>>(&self, message: S) -> anyhow::Result<()> {
//         // Ignore result because it can only fail if there are no receivers remaining.
//         // In that case this shouldn't do anything anyways.
//         let _ = self.broadcast.send(BroadcastPacket::new(
//             Disconnect {
//                 hide_message: false,
//                 message: message.as_ref(),
//             },
//             None,
//         )?);

//         for session in self.list.iter() {
//             session.value().1.cancelled().await;
//         }

//         // Clear to get rid of references, so that the sessions are dropped once they're ready.
//         self.list.clear();

//         Ok(())
//     }

//     /// Returns how many clients are currently connected this tracker.
//     #[inline]
//     pub fn player_count(&self) -> usize {
//         self.list.len()
//     }

//     /// Returns the maximum amount of sessions this tracker will allow.
//     #[inline]
//     pub fn max_player_count(&self) -> usize {
//         SERVER_CONFIG.read().max_players
//     }

//     #[inline]
//     async fn garbage_collector(
//         list: Arc<DashMap<SocketAddr, (mpsc::Sender<MutableBuffer>, Arc<Session>)>>,
//     ) -> ! {
//         let mut interval = tokio::time::interval(GARBAGE_COLLECT_INTERVAL);
//         loop {
//             list.retain(|_, session| -> bool { session.1.is_active() });

//             interval.tick().await;
//         }
//     }
// }