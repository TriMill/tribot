use serenity::prelude::*;
use serenity::model::prelude::*;
use itertools::Itertools;
use crate::state::*;
use crate::utils;

pub type CommandResult = serenity::Result<Option<i32>>;

#[derive(Clone, Copy)]
pub struct Command {
    pub short: &'static str,
    pub aliases: &'static[&'static str],
    pub usage: &'static[&'static str],
    pub description: &'static str,
    pub examples: &'static[&'static str]
}

pub static COMMANDS: &[Command] = &[
    VERSION, SAY, EVAL, ROLL, FLIP, EIGHTBALL, VOTE, POLL, WIKIPEDIA, HELP
];

pub fn dealias<'a>(name: &'a str) -> &'a str {
    match name {
        "calc" | "=" => "eval",
        "dice" => "roll",
        "coinflip" => "flip",
        "?" => "help",
        "eightball" => "8ball",
        "pong" => "ping",
        "wp" | "wiki" => "wikipedia",
        _ => name
    }
}

pub async fn shutdown(ctx: &Context, msg: &Message, state: &mut State, code: i32) -> CommandResult {
    msg.channel_id.say(&ctx.http, ":wave: Cya!").await?;
    state.force_dirty();
    ctx.invisible().await;
    Ok(Some(code))
}

pub async fn activity(ctx: &Context, msg: &Message, rest: &str) -> CommandResult {
    if rest == "reset" {
        ctx.reset_presence().await;
        return Ok(None)
    }
    let idx = rest.find(" ");
    if let Some(i) = idx {
        let kind = &rest[..i];
        let message = &rest[i..];
        let activity = match kind {
            "playing" => Activity::playing(message),
            "listening" => Activity::listening(message),
            "competing" => Activity::competing(message),
            _ => {
                msg.channel_id.say(&ctx.http, ":x: Invalid activity type").await?;
                return Ok(None)
            }
        };
        ctx.set_activity(activity).await;
    } else {
        msg.channel_id.say(&ctx.http, ":x: No activity specified").await?;
    }
    Ok(None)
}

pub async fn status(ctx: &Context, msg: &Message, rest: &str) -> CommandResult {
    match rest {
        "dnd" => ctx.dnd().await,
        "idle" => ctx.idle().await,
        "online" => ctx.online().await,
        "invisible" => ctx.invisible().await,
        "reset" => ctx.reset_presence().await,
        _ => { msg.channel_id.say(&ctx.http, ":x: Invalid status").await?; }
    }
    Ok(None)
}

pub async fn ban_unban(ctx: &Context, msg: &Message, state: &mut State, ban: bool) -> CommandResult {
    if msg.mentions.len() > 0 {
        let user = &msg.mentions[0];
        let result = match ban {
            true => match state.ban(user.id) {
                Ok(()) => format!(":crab: Banned {}#{}", user.name, user.discriminator),
                Err(e) => format!(":x: {}", e),
            },
            false => match state.unban(user.id) {
                Ok(()) => format!(":crab: Unbanned {}#{}", user.name, user.discriminator),
                Err(e) => format!(":x: {}", e),
            }
        };
        msg.channel_id.say(&ctx.http, result).await?;
    } else {
        msg.channel_id.say(&ctx.http, ":x: No user specified").await?;
    }
    Ok(None)
}

pub static VERSION: Command = Command {
    short: "Show version information",
    aliases: &[],
    usage: &["version"],
    description: "Show version information.",
    examples: &[],
};
pub async fn version(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "TriBot v0.1 by TriMill#6898").await?;
    Ok(None)
}

pub static SAY: Command = Command {
    short: "Make the bot say something",
    aliases: &[],
    usage: &["say <message>"],
    description: "Make the bot say something.",
    examples: &[]
};
pub async fn say(ctx: &Context, msg: &Message, rest: &str) -> CommandResult {
    if rest.len() > 0 {
        msg.channel_id.say(&ctx.http, rest).await?;
    }
    Ok(None)
}

pub static PING: Command = Command {
    short: "Ping the bot, showing the ping time",
    aliases: &["pong"],
    usage: &["ping"],
    description: "Ping the bot, showing the time between sending the message and the bot recieving it.",
    examples: &[]
};
pub async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    let now = chrono::offset::Utc::now().naive_utc();
    let mtime = msg.timestamp.naive_utc();
    let diff = (now - mtime).num_milliseconds();
    msg.channel_id.say(&ctx.http, format!(":ping_pong: Pong! in {}ms", diff)).await?;
    Ok(None)
}

pub static EVAL: Command = Command {
    short: "Evaluate an expression",
    aliases: &["calc", "="],
    usage: &["eval <expr>"],
    description: "Evaluate a mathematical expression. Common operators and functions are supported. See <https://docs.rs/meval/0.2.0/meval/#supported-expressions> for more information.",
    examples: &["eval sin(3/4*pi)", "eval 0.5 + sqrt(5)/2", "eval floor(e^3)+1"]
};
pub async fn eval(ctx: &Context, msg: &Message, rest: &str) -> CommandResult {
    match meval::eval_str(rest) {
        Ok(result) => msg.channel_id.say(&ctx.http, format!("Result: `{}`", result)).await?,
        Err(_) => msg.channel_id.say(&ctx.http, ":x: Error parsing expression").await?
    };
    Ok(None)
}


pub static ROLL: Command = Command {
    short: "Roll dice",
    aliases: &["dice"],
    usage: &["roll <dice>"],
    description: "Roll dice. Supports dice with arbitrary sides and constants. See <https://en.wikipedia.org/wiki/Dice_notation> for dice notation information. Total number of dice must not exceed 2048.",
    examples: &["roll 2d6", "roll 1d20-1", "roll 2d8+1d6"]
};
pub async fn roll(ctx: &Context, msg: &Message, rest: &str) -> CommandResult {
    let sort = !rest.starts_with("nosort ");
    let dicestr = match sort {
        false => &rest[7..],
        true => rest,
    }.replace(" ","");
    match utils::roll_dice(&dicestr, sort) {
        Ok(rolls) if rolls.len() > 0 => {
            let result = format!(":game_die: Rolls: `{}` (Sum: **{}**)", 
                rolls.iter()
                    .map(|x| x.to_string())
                    .intersperse(", ".to_owned())
                    .collect::<String>(),
                rolls.iter().fold(0i64, |a, b| a+b)
            );
            if result.len() > 2000 {
                let result = format!(":game_die: Too many rolls to display. Sum: **{}**", 
                    rolls.iter().fold(0i64, |a, b| a+b)
                );
                msg.channel_id.say(&ctx.http, result).await?;
            } else {
                msg.channel_id.say(&ctx.http, result).await?;
            }
        }
        Ok(_) => { msg.channel_id.say(&ctx.http, ":game_die: No dice rolled").await?; },
        Err(e) => { msg.channel_id.say(&ctx.http, format!(":x: Error rolling dice: {}", e)).await?; }
    }
    Ok(None)
}

pub static FLIP: Command = Command {
    short: "Flip coins",
    aliases: &["coinflip"],
    usage: &["flip", "flip <n>"],
    description: "Flip the number of coins specified, or one by default. Number of coins must not exceed 2048.",
    examples: &["flip 6", "flip"]
};
pub async fn flip(ctx: &Context, msg: &Message, rest: &str) -> CommandResult {
    use rand::Rng;
    if rest == "" {
        let side = match rand::thread_rng().gen() {
            true => "Heads",
            false => "Tails"
        };
        msg.channel_id.say(&ctx.http, format!(":coin: {}!", side)).await?;
    } else if let Ok(n) = rest.parse::<u32>() {
        if n > 2048 {
            msg.channel_id.say(&ctx.http, ":x: Too many coins").await?;
        } else {
            let res = {
                let mut rng = rand::thread_rng();
                let (mut heads, mut tails) = (0u32, 0u32);
                for _ in 0..n {
                    match rng.gen() {
                        true => heads += 1,
                        false => tails += 1,
                    }
                }
                format!(":coin: Flipped {} coins, got {} heads and {} tails.", n, heads, tails)
            };
            msg.channel_id.say(&ctx.http, res).await?;
        }
    }
    Ok(None)
}

pub static EIGHTBALL: Command = Command {
    short: "Ask the Magic Eight Ball a question",
    aliases: &["eightball"],
    usage: &["8ball <question>"],
    description: "Ask the Magic Eight Ball a yes/no question, returning a ~~random~~extremely accurate answer",
    examples: &["8ball do people secretly dislike me but are too afraid to tell me so they just pretend they like me"]
};
pub async fn eightball(ctx: &Context, msg: &Message, rest: &str) -> CommandResult {
    if rest == "" {
        msg.channel_id.say(&ctx.http, ":8ball: You must ask the Magic Eight Ball a question.").await?;
    } else {
        msg.channel_id.say(&ctx.http, format!(":8ball: {}", utils::eight_ball())).await?;
    }
    Ok(None)
}

pub static WIKIPEDIA: Command = Command {
    short: "Search Wikipedia",
    aliases: &["wp", "wiki"],
    usage: &["wikipedia <query>"],
    description: "Search Wikipedia. Shows the first result, with a link, text extract, and image if the page has a thumbnail.",
    examples: &[]
};
pub async fn wikipedia(ctx: &Context, msg: &Message, rest: &str) -> CommandResult {
    let result = utils::wikipedia(rest).await;
    if let Ok(res) = result {
        let result = res.clone();
        drop(res);
        msg.channel_id.send_message(&ctx.http, |m| m.embed(|e| {
            e.color(utils::WEB_COLOR);
            e.title(result.title);
            e.description(result.text);
            e.url(result.url);
            e.footer(|x| x.text("From Wikipedia"));
            if let Some(image) = result.image_url {
                e.image(image);
            }
            e
        })).await?;
    } else if let Err(err) = result {
        msg.channel_id.send_message(&ctx.http, |m| m.embed(|e| {
            e.color(utils::WEB_COLOR);
            e.footer(|x| x.text("From Wikipedia"));
            match err {
                utils::WikipediaError::Missing(s) => {
                    if s == "\"No results found\"" {
                        e.title(format!("No results found for \"{}\"", rest));
                    } else {
                        e.title("Error");
                        e.description(s);
                    }
                },
                utils::WikipediaError::Other(s) => {
                    e.title("Wikipedia API error");
                    e.description(s);
                }
            }
            e
        })).await?;
        return Ok(None)
    }
    Ok(None)
}

pub static VOTE: Command = Command {
    short: "Create a poll with two options",
    aliases: &[],
    usage: &["vote <question>"],
    description: "Create a poll with the options :arrow_up: and :arrow_down:. Users may only select one option.",
    examples: &["vote Are waffles better than pancakes?"]
};
pub async fn vote(ctx: &Context, msg: &Message, rest: &str) -> CommandResult {
    let vote_msg = msg.channel_id.send_message(&ctx.http, |m| m.embed(|e| {
        e.footer(|f| f.text(format!("{}#{}", msg.author.name, msg.author.discriminator)));
        e.color(utils::POLL_COLOR);
        e.title(rest);
        e
    })).await?;
    vote_msg.react(&ctx.http, '\u{2B06}').await?;
    vote_msg.react(&ctx.http, '\u{2B07}').await?;
    Ok(None)
}

pub static POLL: Command = Command {
    short: "Create a poll with multiple options",
    aliases: &[],
    usage: &["poll <question>;<options...>"],
    description: "Create a poll with multiple options. Arguments are separated by semicolons, and the first argument is the poll question. Number of options must be between 1 and 9 inclusive. Users may only select one option.",
    examples: &["poll Best breakfast food; Waffles; Pancakes; Toast"]
};
pub async fn poll(ctx: &Context, msg: &Message, rest: &str) -> CommandResult {
    let parts = rest.split(";").collect::<Vec<&str>>();
    if parts.len() < 2 {
        msg.channel_id.say(&ctx.http, ":x: Not enough arguments. See `;help poll`.").await?;
        return Ok(None)
    } else if parts.len() > 10 {
        msg.channel_id.say(&ctx.http, ":x: Too many arguments. See `;help poll`.").await?;
        return Ok(None)
    }
    let question = parts[0];
    let options = &parts[1..];
    let body = options.iter()
        .enumerate()
        .map(|(i,x)| format!("{}: {}", utils::NUM_EMOJIS[i+1], x.trim()))
        .collect::<Vec<String>>()
        .join("\n");
    let poll_msg = msg.channel_id.send_message(&ctx.http, |m| m.embed(|e| {
        e.footer(|f| f.text(format!("{}#{}", msg.author.name, msg.author.discriminator)));
        e.color(utils::POLL_COLOR);
        e.title(question);
        e.description(body);
        e
    })).await?;
    for i in 1..(options.len() + 1) {
        poll_msg.react(&ctx.http, 
            ReactionType::Unicode(utils::NUM_EMOJIS[i].to_owned())).await?;
    }
    Ok(None)
}

pub static HELP: Command = Command {
    short: "Show help",
    aliases: &["?"],
    usage: &["help", "help <cmd>"],
    description: "Show help for a specific command, or show a list of commands if no command is specified",
    examples: &["help", "help roll", "help help"]
};
pub async fn send_help(ctx: &Context, msg: &Message) -> CommandResult {
    let mut body = String::new();
    for cmd in crate::commands::COMMANDS {
        let usage = cmd.usage[0];
        let short = cmd.short;
        body += &format!("`{}`: {}\n", usage, short);
    }
    msg.channel_id.send_message(&ctx.http, |m| m.embed(|e| {
        e.title("TriBot Help");
        e.color(utils::HELP_COLOR);
        e.description(body);
        e
    })).await?;
    Ok(None)
}

pub async fn send_help_command(ctx: &Context, msg: &Message, rest: &str) -> CommandResult {
    let cmd_name = dealias(rest);
    let cmd: Command = match cmd_name {
        "version" => VERSION,
        "say" => SAY,
        "ping" => PING,
        "roll" => ROLL,
        "flip" => FLIP,
        "eval" => EVAL,
        "help" => HELP,
        "8ball" => EIGHTBALL,
        "wikipedia" => WIKIPEDIA,
        "vote" => VOTE,
        "poll" => POLL,
        _ => {
            msg.channel_id.say(&ctx.http, format!(":x: Unknown command `{}`", rest)).await?;
            return Ok(None)
        }
    };
    msg.channel_id.send_message(&ctx.http, |m| m.embed(|e| {
        e.title(format!("Help for command `{}`", cmd_name));
        e.color(utils::HELP_COLOR);
        if cmd.aliases.len() > 0 {
            e.field("Aliases", cmd.aliases
                .iter()
                .map(|x| format!("`{}`", x))
                .collect::<Vec<String>>()
                .join(" | "), false);
        }
        e.field("Usage", cmd.usage
            .iter()
            .map(|x| format!("`{}`", x))
            .collect::<Vec<String>>()
            .join(" | "), false);
        e.field("Description", cmd.description, false);
        if cmd.examples.len() > 0 {
            e.field("Examples", cmd.examples
                .iter()
                .map(|x| format!("`{}`", x))
                .collect::<Vec<String>>()
                .join("\n"), false);
        }
        e
    })).await?;
    Ok(None)
}
