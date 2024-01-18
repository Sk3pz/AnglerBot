use std::{env, sync::Arc};

use serenity::{all::{GatewayIntents, Message, ResumedEvent, Ready, Interaction, CommandInteraction},
 Client, async_trait, client::{EventHandler, Context}, prelude::TypeMapKey, gateway::ShardManager,
               builder::{CreateInteractionResponse, CreateInteractionResponseMessage}};
use serenity::all::{ActivityData, Command, CreateCommand, OnlineStatus};
use crate::commands::fish::{catch, FishCatch};
use crate::data::fish::{Fish, FishData, FishRarity};
use crate::data::userfile::set_userfile_casting_false;

pub mod logging;
pub mod data;

mod commands;

// todo: ideas
//   - Leveling system
//   - A bait system to increase the chances of catching fish

pub const SKEPZ_ID: u64 = 318884828508454912;

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<ShardManager>;
}

pub fn stop_users_fishing() {
    // loop through all the files in /data/users
    let guild_paths = std::fs::read_dir("./data/guilds").unwrap();
    for path in guild_paths {
        // get all guild directories
        let path = path.unwrap().path();
        let path = path.to_str().unwrap();
        let user_paths = std::fs::read_dir(format!("{}/users/", path)).unwrap();
        for user_path in user_paths {
            let user_path = user_path.unwrap().path();
            let user_path = user_path.to_str().unwrap();
            set_userfile_casting_false(user_path.to_string());
        }
    }
}

pub async fn command_response<S: Into<String>>(ctx: &Context, command: &CommandInteraction, msg: S) {
    let data = CreateInteractionResponseMessage::new().content(msg.into());
    let builder = CreateInteractionResponse::Message(data);
    if let Err(err) = command.create_response(&ctx.http, builder).await {
       nay!("Failed to respond to command: {}", err)
    }
}

pub async fn register_command(ctx: &Context, cmd: CreateCommand) {
    if let Err(e) = Command::create_global_command(&ctx.http, cmd).await {
        nay!("Failed to register a command: {}", e);
    }
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        // Ignore messages from bots
        if msg.author.bot {
            return;
        }

        if msg.author.id != SKEPZ_ID {
            return;
        }

        if msg.content.starts_with("!spawn") {
            let args = msg.content.split(' ').collect::<Vec<&str>>();
            if args.len() < 4 {
                return;
            }
            let rarity = FishRarity::from_string(args[1]).unwrap();
            // get args[2] as an f32
            let weight = args[2].parse::<f32>().unwrap();
            // get the fish
            let fishdata = FishData::load();
            let fish = fishdata.fish_type_by_name(args[3..].join(" ")).unwrap();

            let fish = Fish {
                fish_type: fish.clone(),
                rarity: rarity.clone(),
                weight,
            };

            catch(ctx.http, msg.channel_id, msg.author.id, msg.guild_id.unwrap(), FishCatch {
                user_file: data::userfile::read_userfile(&msg.author.id, msg.guild_id.unwrap().get()),
                fish,
                will_catch: true,
                override_special: true,
            }).await;

        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        let config = data::config::Config::load();
        // remove all global commands
        //Command::set_global_commands(&ctx.http, Vec::new()).await.expect("Failed to remove global commands");
        // create the commands
        register_command(&ctx, commands::fish::register(config.motd.clone())).await;
        register_command(&ctx, commands::shop::register()).await;
        register_command(&ctx, commands::buy::register()).await;
        register_command(&ctx, commands::balance::register()).await;
        register_command(&ctx, commands::info::register()).await;
        register_command(&ctx, commands::rod_info::register()).await;
        register_command(&ctx, commands::bestiary::register()).await;

        yay!("{} is connected! {}", ready.user.name, config.motd);
        if !config.debug_mode {
            ctx.set_presence(Some(ActivityData::playing("/fish")), OnlineStatus::Online);
        } else {
            ctx.set_presence(Some(ActivityData::custom("Under development")), OnlineStatus::Online);
        }
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        let config = data::config::Config::load();
        stop_users_fishing();
        yay!("{}", config.motd);
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let command_name = command.data.name.as_str();
            let sender = &command.user;
            let guild = command.guild_id;
            if guild.is_none() {
                command_response(&ctx, &command, "You must be in a server to do this!").await;
                return;
            }
            let guild_id = guild.unwrap();
            let command_options = &command.data.options();
            let channel = &command.channel_id;

            let config = data::config::Config::load();

            if config.debug_mode && sender.id != SKEPZ_ID {
                command_response(&ctx, &command, "The pond is being restocked, please try again later! (Under maintenance)").await;
                return;
            }

            match command_name {
                "fish" => {
                    command_response(&ctx, &command, commands::fish::run(&ctx, channel, sender, &guild_id)).await
                }
                "shop" => {
                    commands::shop::run(&ctx, &command).await
                }
                "buy" => {
                    command_response(&ctx, &command, commands::buy::run(command_options, sender, &guild_id)).await
                }
                "rod" => {
                    commands::rod_info::run(command_options, &ctx, &command).await;
                }
                "balance" => {
                    command_response(&ctx, &command, commands::balance::run(&sender.id, &guild_id)).await
                }
                "info" => {
                    commands::info::run(&ctx, &command, sender, &guild_id).await;
                }
                "bestiary" => {
                    commands::bestiary::run(&ctx, &command, sender, &guild_id).await;
                }
                _ => {
                    command_response(&ctx, &command, "Unknown command").await
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    yay!("Angler Bot starting up!");

    dotenv::dotenv().expect("Failed to load .env file");

    let Ok(token) = env::var("DISCORD_TOKEN") else {
        nay!("DISCORD_TOKEN not found in environment");
        return;
    };

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Build the client
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
    }

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        if let Err(e) = tokio::signal::ctrl_c().await {
            nay!("Failed to listen for ctrl-c: {}", e);
            return;
        }

        // loop through all the files in /data/users
        stop_users_fishing();

        shard_manager.shutdown_all().await;
    });

    if let Err(err) = client.start().await {
        nay!("Angler Bot failed to cast: {}", err);
    }
}
