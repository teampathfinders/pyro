use std::time::Duration;

use bytes::BytesMut;
use common::{VResult, WriteExtensions};

use crate::network::Encodable;

use super::GamePacket;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TitleAction {
    Clear,
    Reset,
    SetTitle,
    SetSubtitle,
    SetActionBar,
    SetDurations,
    TitleTextObject,
    SubtitleTextObject,
    ActionBarTextObject,
}

/// Sets a title for the client.
/// This is basically the same as the /title command in vanilla Minecraft.
#[derive(Debug)]
pub struct SetTitle {
    /// Title operation to perform.
    pub action: TitleAction,
    /// Text to display.
    pub text: String,
    /// Fade in duration (in ticks).
    pub fade_in_duration: i32,
    /// How long the title remains on screen (in ticks).
    pub remain_duration: i32,
    /// Fade out duration (in ticks).
    pub fade_out_duration: i32,
    /// XUID of the client.
    pub xuid: String,
    /// Either an uint64 or an empty string.
    pub platform_online_id: String,
}

impl GamePacket for SetTitle {
    const ID: u32 = 0x58;
}

impl Encodable for SetTitle {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::new();

        buffer.put_var_i32(self.action as i32);
        buffer.put_string(&self.text);
        buffer.put_var_i32(self.fade_in_duration);
        buffer.put_var_i32(self.remain_duration);
        buffer.put_var_i32(self.fade_out_duration);
        buffer.put_string(&self.xuid);
        buffer.put_string(&self.platform_online_id);

        Ok(buffer)
    }
}