use log::{error, info, warn};
use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    framework::standard::{
        macros::{group, hook, help},
        StandardFramework, Args, HelpOptions, CommandGroup, CommandResult, help_commands::plain,
    },
    http::CacheHttp,
    model::{id::GuildId, prelude::*},
    prelude::*,
};

use std::{
    collections::{HashMap, HashSet},
    env, process,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use vc_notifier::{commands::setting::*, config::*};

#[group]
#[commands(settings)]
struct VCNotifier;

#[hook]
async fn before_hook(_: &Context, msg: &Message, _: &str) -> bool {
    if let Some(_) = msg.guild_id {
        true
    } else {
        false
    }
}

#[hook]
async fn unrecognized_command_hook(ctx: &Context, msg: &Message, cmd_name: &str) {
    warn!("{} is called!", cmd_name);
    if let Some(_) = msg.guild_id {
        msg.reply(ctx.http(), format!("{} is not existed!", cmd_name))
            .await
            .expect("failed reply");
    }
}

struct Handler {
    is_locking: Arc<AtomicBool>,
}

#[help]
async fn vcn_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>
) -> CommandResult {
    let _ = plain(context, msg, args, &help_options, groups, owners).await;
    Ok(())
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, r: Ready) {
        info!("{} has been connected!", r.user.name);
    }

    async fn guild_create(&self, ctx: Context, guild: Guild, is_new: bool) {
        {
            let data = ctx.data.read().await;
            let gid = &guild.id;
            if is_new {
                let m_lock = data.get::<VCNSettings>().unwrap();
                let mut w_m_lock = m_lock.write().await;
                w_m_lock.insert(*gid, _Settings::new());
            } else {
                // TODO: get settings from database (need to create database)
                let m_lock = data.get::<VCNSettings>().unwrap();
                let mut w_m_lock = m_lock.write().await;
                w_m_lock.entry(*gid).or_insert(_Settings::new());
            }
            let mut vc_members: u64 = 0;
            for (_id, chan) in guild.channels {
                match chan.kind {
                    ChannelType::Voice => {
                        // **Note** this logic will be replaced when channel is private
                        let current_members = chan
                            .members(ctx.cache.clone())
                            .await
                            .unwrap_or_else(|err| {
                                warn!("{:#?}", err);
                                vec![]
                            })
                            .len() as u64;
                        vc_members += current_members;
                    }
                    _ => continue,
                }
            }
            {
                let m_lock = data.get::<VCCounts>().unwrap();
                let mut w_m_lock = m_lock.write().await;
                w_m_lock.insert(*gid, vc_members);
            }
        }
    }

    async fn voice_state_update(
        &self,
        ctx: Context,
        guild_id: Option<GuildId>,
        old: Option<VoiceState>,
        _new: VoiceState,
    ) {
        let gid = guild_id.unwrap_or_else(|| {
            warn!("cannot read guild_id, so set to `GuildId(0)`");
            GuildId(0)
        });
        let settings_map_lock = {
            let r_data = ctx.data.read().await;
            r_data.get::<VCNSettings>().unwrap().clone()
        };
        let settings_map = settings_map_lock.read().await;
        let settings = settings_map.get(&gid);

        let settings = if let Some(s) = settings {
            s.clone()
        } else {
            return;
        };

        if !settings.enable {
            return;
        }

        let send_channel = if let Some(chan) = settings.hooked_channel {
            chan
        } else {
            warn!("hooked_channel is `None`. Skipped process.");
            return;
        };

        let ctx = Arc::new(ctx);

        if let Some(_) = old {
            // somebody left voice channel
            if _read_vc_members(&ctx, &gid).await == 1 {
                if !self.is_locking.load(Ordering::Relaxed) {
                    send_channel
                        .send_message(&ctx.http, |m| {
                            m.content(settings.leave_message.as_str());
                            m
                        })
                        .await
                        .expect("");
                }
            }
            _decrement_vc_members(&ctx, &gid).await;
        } else {
            // somebody joined voice channel
            let mut trigger = false;
            if _read_vc_members(&ctx, &gid).await == 0 {
                trigger = true;
                _increment_vc_members(&ctx, &gid).await;
                if !self.is_locking.load(Ordering::Relaxed) {
                    self.is_locking.swap(true, Ordering::Relaxed);
                    let ctx_clone = Arc::clone(&ctx);
                    let locking_clone = Arc::clone(&self.is_locking);
                    tokio::spawn(async move {
                        tokio::time::sleep(Duration::from_secs(settings.duration)).await;
                        if _read_vc_members(&ctx_clone, &gid).await != 0 {
                            send_channel
                                .send_message(&ctx_clone.http, |m| {
                                    m.content(settings.join_message.as_str());
                                    m
                                })
                                .await
                                .expect("");
                        }
                        locking_clone.swap(false, Ordering::Relaxed);
                    });
                }
            }
            if !trigger {
                _increment_vc_members(&ctx, &gid).await;
            }
        };
    }
}

async fn _read_vc_members(ctx: &Context, guild_id: &GuildId) -> u64 {
    let vc_members_map_lock = {
        let r_data = ctx.data.read().await;
        r_data.get::<VCCounts>().unwrap().clone()
    };
    let vc_members_map = vc_members_map_lock.read().await;
    let vc_members = vc_members_map.get(guild_id);

    *vc_members.unwrap()
}

async fn _increment_vc_members(ctx: &Context, guild_id: &GuildId) {
    let vc_members_map_lock = {
        let r_data = ctx.data.read().await;
        r_data.get::<VCCounts>().unwrap().clone()
    };
    let mut vc_members_map = vc_members_map_lock.write().await;
    let vc_members = vc_members_map.get_mut(guild_id).unwrap();

    *vc_members += 1;
}

async fn _decrement_vc_members(ctx: &Context, guild_id: &GuildId) {
    let vc_members_map_lock = {
        let r_data = ctx.data.read().await;
        r_data.get::<VCCounts>().unwrap().clone()
    };
    let mut vc_members_map = vc_members_map_lock.write().await;
    let vc_members = vc_members_map.get_mut(guild_id).unwrap();

    *vc_members -= 1;
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("vn=="))
        .before(before_hook)
        .unrecognised_command(unrecognized_command_hook)
        .group(&VCNOTIFIER_GROUP)
        .help(&VCN_HELP);

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("Need `DISCORD_TOKEN` in env`");
    let mut client = Client::builder(token)
        .event_handler(Handler {
            is_locking: Arc::new(AtomicBool::new(false)),
        })
        .framework(framework)
        .await
        .expect("Error creating client");

    {
        let mut w_data = client.data.write().await;
        w_data.insert::<VCNSettings>(Arc::new(RwLock::new(HashMap::<GuildId, _Settings>::new())));
        w_data.insert::<VCCounts>(Arc::new(RwLock::new(HashMap::<GuildId, u64>::new())));
    }

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        error!("An error occurred while running the client: {:?}", why);
        process::exit(1);
    }
}
