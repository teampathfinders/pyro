pub use biome_definition_list::*;
pub use cache_status::*;
pub use chunk_radius_reply::*;
pub use chunk_radius_request::*;
pub use client_to_server_handshake::*;
pub use creative_content::*;
pub use disconnect::*;
pub use login::*;
pub use network_settings::*;
pub use online_ping::*;
pub use online_pong::*;
pub use packet::*;
pub use play_status::*;
pub use request_network_settings::*;
pub use resource_pack_client_response::*;
pub use resource_pack_stack::*;
pub use resource_packs_info::*;
pub use server_to_client_handshake::*;
pub use start_game::*;
pub use traits::*;
pub use violation_warning::*;
pub use level_chunk::*;
pub use interact::*;
pub use text::*;
pub use set_time::*;
pub use available_commands::*;
pub use play_sound::*;
pub use show_profile::*;
pub use set_player_gamemode::*;
pub use set_health::*;
pub use mob_effect::*;
pub use show_credits::*;
pub use set_difficulty::*;
pub use set_commands_enabled::*;
pub use tick_sync::*;
pub use add_painting::*;

mod add_painting;
mod tick_sync;
mod set_commands_enabled;
mod set_difficulty;
mod show_credits;
mod mob_effect;
mod set_health;
mod set_player_gamemode;
mod show_profile;
mod play_sound;
mod available_commands;
mod set_time;
mod text;
mod interact;
mod level_chunk;
mod biome_definition_list;
mod cache_status;
mod chunk_radius_reply;
mod chunk_radius_request;
mod client_to_server_handshake;
mod creative_content;
mod disconnect;
mod login;
mod network_settings;
mod online_ping;
mod online_pong;
mod packet;
mod play_status;
mod request_network_settings;
mod resource_pack_client_response;
mod resource_pack_stack;
mod resource_packs_info;
mod server_to_client_handshake;
mod start_game;
mod traits;
mod violation_warning;

/// ID of Minecraft game packets.
pub const GAME_PACKET_ID: u8 = 0xfe;
/// Semver version that this server supports.
pub const CLIENT_VERSION_STRING: &str = "1.19.60";
/// Protocol version that this server supports.
pub const NETWORK_VERSION: u32 = 567;
