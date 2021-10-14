use log::{debug, info};
use serenity::{framework::standard::{Args, CommandResult, macros::command}, http::CacheHttp, model::{channel::{Message}, id::ChannelId}, prelude::*};

use crate::config::{VCNSettings};

#[command]
#[sub_commands(set_enable, set_disable, hook_channel, show_settings, set_duration, set_join_message, set_leave_message)]
pub(crate) async fn settings(_ctx: &Context, _msg: &Message, _args: Args) -> CommandResult {
    // TODO: show help etc...

    Ok(())
}

#[command("enable")]
async fn set_enable(ctx: &Context, msg: &Message, _: Args) -> CommandResult {
    info!("enable command was called");
    {
        let gid = if let Some(guild) = msg.guild_id {
            guild
        } else {
            return Ok(())
        };
        let data = ctx.data.read().await;
        let settings_map = data.get::<VCNSettings>().unwrap();
        let mut settings_map_lock = settings_map.write().await;
        let settings = settings_map_lock.get_mut(&gid).unwrap();
        if settings.enable {
            msg.reply(ctx.http(), "The bot is already enabled!").await?;
            return Ok(())
        }
        if let Some(_) = settings.hooked_channel {
            settings.enable = true;
            debug!("enabled bot");
            msg.reply(ctx.http(), "The bot has been enabled!").await?;
        } else {
            msg.reply(ctx.http(), "Before starting the bot, you need to hook the channel to send messages.").await?;
        }
    }
    Ok(())
}

#[command("disable")]
async fn set_disable(ctx: &Context, msg: &Message, _: Args) -> CommandResult {
    info!("disable command was called!");
    {
        let gid = if let Some(guild) = msg.guild_id {
            guild
        } else {
            return Ok(())
        };
        let data = ctx.data.read().await;
        let settings_map = data.get::<VCNSettings>().unwrap();
        let mut settings_map_lock = settings_map.write().await;
        let settings = settings_map_lock.get_mut(&gid).unwrap();
        if !settings.enable {
            msg.reply(ctx.http(), "The bot is already disabled!").await?;
        } else {
            settings.enable = false;
            debug!("disabled bot");
            msg.reply(ctx.http(), "The bot has been disabled!").await?;
        }
    }
    Ok(())
}

#[command("hook")]
async fn hook_channel(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    info!("hook command was called!");
    let args_len = args.len();
    if args_len != 1 {
        msg.reply(ctx.http(), format!("Need 1 argument, but {} was given.", args_len.to_string())).await?;
        return Ok(())
    }

    // TODO: add error handling in `main.rs`.

    let channel_id = args.single::<ChannelId>()?;

    {
        let data = ctx.data.read().await;
        let settings_map = data.get::<VCNSettings>().unwrap();
        let mut settings_map_lock = settings_map.write().await;
        let settings = settings_map_lock.get_mut(&msg.guild_id.unwrap()).unwrap();
        settings.hooked_channel = Some(channel_id);
    }
    msg.reply(ctx.http(), format!("channel is successfully hooked to <#{}>!", channel_id.to_string())).await?;

    Ok(())
}

#[command("show")]
async fn show_settings(ctx: &Context, msg: &Message, _: Args) -> CommandResult {
    info!("show command was called!");

    let settings_map = {
        let data = ctx.data.read().await;
        data.get::<VCNSettings>().unwrap().clone()
    };
    let settings_map_lock = settings_map.read().await;
    let settings = settings_map_lock.get(&msg.guild_id.unwrap()).unwrap();

    msg.reply(ctx.http(), format!("```{:#?}```", settings)).await?;

    Ok(())
}

#[command("duration")]
async fn set_duration(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    info!("duration command was called!");

    let args_len = args.len();

    if args_len != 1 {
        msg.reply(ctx.http(), format!("Need 1 argument, but {} was given.", args_len.to_string())).await?;
        return Ok(())
    }

    let duration_time = args.single::<u64>()?;

    {
        let settings_map = {
            let data = ctx.data.read().await;
            data.get::<VCNSettings>().unwrap().clone()
        };
        let mut settings_map_lock = settings_map.write().await;
        let settings = settings_map_lock.get_mut(&msg.guild_id.unwrap()).unwrap();
        settings.duration = duration_time;
    }
    msg.reply(ctx.http(), format!("duration was set to `{}`!", duration_time.to_string())).await?;

    Ok(())
}

#[command("join_message")]
async fn set_join_message(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    info!("join_message command was called!");

    let args_len = args.len();

    if args_len != 1 {
        msg.reply(ctx.http(), format!("Need 1 argument, but {} was given.", args_len.to_string())).await?;
        return Ok(())
    }

    let mes = args.single::<String>()?;

    {
        let settings_map = {
            let data = ctx.data.read().await;
            data.get::<VCNSettings>().unwrap().clone()
        };
        let mut settings_map_lock = settings_map.write().await;
        let settings = settings_map_lock.get_mut(&msg.guild_id.unwrap()).unwrap();
        settings.join_message = mes;
    }

    msg.reply(ctx.http(), "successfully set join message!").await?;

    Ok(())
}

#[command("leave_message")]
async fn set_leave_message(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    info!("leave_message command was called!");

    let args_len = args.len();

    if args_len != 1 {
        msg.reply(ctx.http(), format!("Need 1 argument, but {} was given.", args_len.to_string())).await?;
        return Ok(())
    }

    let mes = args.single::<String>()?;

    {
        let settings_map = {
            let data = ctx.data.read().await;
            data.get::<VCNSettings>().unwrap().clone()
        };
        let mut settings_map_lock = settings_map.write().await;
        let settings = settings_map_lock.get_mut(&msg.guild_id.unwrap()).unwrap();
        settings.leave_message = mes;
    }

    msg.reply(ctx.http(), "successfully set leave message!").await?;

    Ok(())
}
