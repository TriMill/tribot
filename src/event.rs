use log::{debug, info, warn, error};
use serenity::{
    async_trait,
    model::{
        prelude::*,
    },
    client::bridge::gateway::ShardManager,
    prelude::*,
    utils::Colour
};
use crate::state::*;
use crate::commands;
use crate::utils;

pub struct ShardManagerKey;
impl TypeMapKey for ShardManagerKey {
    type Value = std::sync::Arc<Mutex<ShardManager>>;//std::sync::Arc<Mutex<ShardManager>>;
}

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let mut exitcode = None;
        if !msg.author.bot && msg.content.starts_with(";") {
            let mut data = ctx.data.write().await;
            let state = match data.get_mut::<State>() {
                Some(x) => x,
                None => {
                    error!("Could not load state data");
                    return
                }
            };
            let banned = state.is_banned(msg.author.id);
            if !banned {
                let result = run_command(&ctx, &msg, state).await;
                match result {
                    Err(e) => warn!("Error running command: {:?}", e),
                    Ok(code) => exitcode = code
                }
                match state.save_if_dirty() {
                    Ok(true) => info!("State saved"),
                    Ok(false) => (),
                    Err(e) => error!("Attempt to save dirty state failed: {:?}", e)
                }
            }
        }
        if let Some(code) = exitcode {
            info!("Command requested exit with code {}", code);
            let mut data = ctx.data.write().await;
            let shardmanager = match data.get_mut::<ShardManagerKey>() {
                Some(s) => s,
                None => {
                    error!("Could not get shard manager, force exiting");
                    std::process::exit(1);
                }
            };
            shardmanager.lock().await.shutdown_all().await;
            std::process::exit(code);
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        ctx.online().await;
        ctx.set_activity(Activity::playing("your mother")).await;
        info!("Ready");
        info!("Guild count: {}", ready.guilds.len());
    }

    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        let bot_user = ctx.http.get_current_user().await.unwrap();
        if reaction.user_id.unwrap() == bot_user.id {
            return 
        }
        let reactor = reaction.user_id.unwrap().to_user(&ctx.http).await;
        if let Err(e) = reactor {
            warn!("Could not get reactor user: {:?}", e);
            return
        }
        let reactor = reactor.unwrap();
        if reactor.bot {
            return
        }
        let message = ctx.http.get_message(reaction.channel_id.into(), reaction.message_id.into()).await;
        if let Err(e) = message {
            warn!("Could not get reaction message: {:?}", e);
            return
        }
        let message = message.unwrap();
        if message.author.id == bot_user.id
        && message.embeds.len() > 0
        && message.embeds[0].colour == Colour::from(utils::POLL_COLOR) {
            let user_id = reactor.id;
            let my_emoji = reaction.emoji;
            let emojis = message.reactions.into_iter()
                .map(|x| x.reaction_type)
                .filter(|x| x != &my_emoji)
                .collect::<Vec<ReactionType>>();
            for emoji in emojis {
                let reactions = ctx.http.get_reaction_users(
                    message.channel_id.into(), message.id.into(),
                    &emoji, 50, None).await;
                match reactions {
                    Ok(r) if r.iter().find(|x| x.id == user_id).is_some() => {
                        let res = ctx.http.delete_reaction(
                            message.channel_id.into(), message.id.into(),
                            Some(user_id.into()), &emoji).await;
                        if let Err(e) = res {
                            warn!("Could not remove reaction: {:?}", e);
                        } else {
                            debug!("Removed reaction");
                        }
                    },
                    Ok(_) => (),
                    Err(e) => warn!("Error retrieving reactions: {:?}", e)
                }
            }
        }
    }
}

pub const CMD_FORBID: &[char] = &[
    '(', ')', '[', ']', '{', '}', ';', '.', ',', ':'
];

pub async fn run_command(ctx: &Context, msg: &Message, state: &mut State) -> commands::CommandResult {
    use crate::commands::*;
    let content = msg.content.trim();
    let idx = content.find(" ").unwrap_or(content.len());
    let cmd = &content[1..idx].trim();
    if cmd.contains(CMD_FORBID) {
        return Ok(None)
    }
    debug!("Command '{}' from {}#{}", cmd, msg.author.name, msg.author.discriminator);
    let rest = &content[idx..].trim().to_owned();
    let sender_admin = state.is_admin(msg.author.id);
    match dealias(cmd) {
        "force_save" if sender_admin => {state.force_dirty(); Ok(None)},
        "stop" if sender_admin => shutdown(ctx, msg, state, 0).await,
        "restart" if sender_admin => shutdown(ctx, msg, state, 5).await,
        "ban" if sender_admin => ban_unban(ctx, msg, state, true).await,
        "unban" if sender_admin => ban_unban(ctx, msg, state, false).await,
        "activity" if sender_admin => activity(ctx, msg, rest).await,
        "status" if sender_admin => status(ctx, msg, rest).await,
        "add" if sender_admin => add_cmd(rest, state).await,
        "rm" if sender_admin => rm_cmd(rest, state).await,
        "ban" | "unban" | "force_save" | "stop" | "restart"
            | "activity" | "status" | "add" | "rm"
            => no_perms(ctx, msg).await,
        "version" => version(ctx, msg).await,
        "say" => say(ctx, msg, rest).await,
        "ping" => ping(ctx, msg).await,
        "count" => count(ctx, msg, state).await,
        "counttop" => counttop(ctx, msg, state).await,
        "roll" => roll(ctx, msg, rest).await,
        "8ball" => eightball(ctx, msg, rest).await,
        "wikipedia" => wikipedia(ctx, msg, rest).await,
        "xkcd" => xkcd(ctx, msg, rest).await,
        "meme" => meme(ctx, msg, rest).await,
        "flip" => flip(ctx, msg, rest).await,
        "eval" => eval(ctx, msg, rest).await,
        "vote" => vote(ctx, msg, rest).await,
        "poll" => poll(ctx, msg, rest).await,
        "help" if rest == "" => send_help(ctx, msg).await,
        "help" => send_help_command(ctx, msg, rest).await,
        _ => match state.run_custom_cmd(cmd) {
            Some(x) => {
                msg.channel_id.say(&ctx.http, format!("{}: {}", msg.author.name, x)).await?;
                Ok(None)
            }
            None => bad_command(ctx, msg).await
        }
    }
}

pub async fn no_perms(ctx: &Context, msg: &Message) -> commands::CommandResult {
    msg.channel_id.say(&ctx.http, ":x: You aren't authorised to do that!").await?;
    Ok(None)
}

pub async fn bad_command(ctx: &Context, msg: &Message) -> commands::CommandResult {
    msg.channel_id.say(&ctx.http, ":x: Invalid command. Use `;help` for help.").await?;
    Ok(None)
}

