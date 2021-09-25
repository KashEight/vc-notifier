use std::{collections::HashMap, sync::Arc};

use serenity::{model::id::{ChannelId, GuildId}, prelude::{RwLock, TypeMapKey}};

#[derive(Debug, Clone)]
pub struct _Settings {
    pub enable: bool,
    pub duration: u64,
    pub join_message: String,
    pub leave_message: String,
    pub hooked_channel: Option<ChannelId>,
}

impl _Settings {
    pub fn new() -> Self {
        Self {
            enable: false,
            duration: 10,
            join_message: "Someone joined voice channel.".to_string(),
            leave_message: "All participents left from voice channel.".to_string(),
            hooked_channel: None,
        }
    }
}

pub struct VCNSettings;

impl TypeMapKey for VCNSettings {
    type Value = Arc<RwLock<HashMap<GuildId, _Settings>>>;
}
pub struct VCCounts;

impl TypeMapKey for VCCounts {
    type Value = Arc<RwLock<HashMap<GuildId, u64>>>;
}
