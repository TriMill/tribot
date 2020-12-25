# TriBot
Custom Discord bot in Rust. The prefix is `;`.

## Features
 - Create polls and votes
 - Evaluate mathematical expressions
 - Roll dice, flip coins, and Magic 8-Ball
 - Search Wikipedia
 - View xkcd comics
 - Competitive counting with a global leaderboard
 - Descriptive help for each command

## Permissions
TriBot requires the `Manage Messages` permission in order to remove double reactions from polls. Although the bot will still function without it, it will complain to you in the logs.

## Commands
For more information, use the `;help` command
| Command     | Description                         |
| `version`   | Show the bot version                |
| `say`       | Make the bot say something          |
| `ping`      | Show the ping time for the bot      |
| `count`     | Increase your count by 1            |
| `counttop`  | View the top players by count       |
| `eval`      | Evaluate an expression              |
| `roll`      | Roll dice                           |
| `flip`      | Flip coins                          |
| `8ball`     | Ask the Magic Eight Ball a question |
| `vote`      | Create a poll with two options      |
| `poll`      | Create a poll with multiple options |
| `wikipedia` | Search Wikipedia                    |
| `xkcd`      | View an xkcd comic                  |
| `help`      | Show help                           |

## Building and running
After installing [Rust](https://www.rust-lang.org/), `git clone` this repository and run `cargo build`. To run the bot, set the following environment variables:

 - `DISCORD_BOT_TOKEN`, your bot's token
 - `DISORD_STATE_FILE`, the path to the file containing the bot's JSON data. Copy `initial_state.json` and add your user ID into the `admins` array.

Optionally, set the following variables to enable the `;meme` command (using the ImgFlip API):
 - `IMGFLIP_USER`, your ImgFlip account's username
 - `IMGFLIP_PASSWD`, the account password

The bot includes the `;restart` command to restart it in place. To use this, create the folder `secrets` in the bot's root directory. Inside it, create the following files:

 - `envars.sh`: a bash script that sets the environment variables
 - `state.json`: the bot's JSON data file (as described above)

Then, build the bot in release mode with `cargo build --release` and run `scripts/run_bot.sh` to start the bot.

## Admin commands
The following commands are available to bot admins. Admins can only be added and removed by editing the JSON data file directly, and this should only be done while the bot is stopped.
| Command                     | Description                                                                                                                                       |
| `force_save`                | Force the bot to overwrite its data file                                                                                                          |
| `stop`                      | Stop the bot                                                                                                                                      |
| `restart`                   | Restart the bot (only works when using the `run_bot.sh script`                                                                                    |
| `ban <@user>`               | Ban a user                                                                                                                                        |
| `unban <@user>`             | Unban a user                                                                                                                                      |
| `activity <type> <message>` | Change the bot's activity message. `type` must be one of `playing`, `listening`, or `competing`. Use `activity reset` to clear the message.       |
| `status <status>`           | Change the bot's status. `status` must be one of `online`, `idle`, `dnd`, or `invisible`. `status reset` has the same effect as `activity reset`. |
| `add <command> <message>`   | Add a custom command. When the command is run, the message will be sent.                                                                          |
| `rm <command>`              | Remove a custom command.                                                                                                                          |
