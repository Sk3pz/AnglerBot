use serenity::all::{Colour, CommandInteraction, Context, CreateAttachment, CreateCommand, CreateEmbed,
                    CreateInteractionResponse, CreateInteractionResponseMessage, GuildId, Timestamp, User};
use crate::data::fish::FishData;
use crate::data::userfile::read_userfile;
use crate::nay;

pub async fn run(ctx: &Context, cmd: &CommandInteraction, sender: &User, guild: &GuildId) {
    let user_data = read_userfile(&sender.id, guild.get());

    let fish_data = FishData::load();

    let rod = user_data.get_rod();

    // create the embedded message
    let embed = CreateEmbed::new()
        .title(format!("{}'s Info", sender.global_name.clone().unwrap_or(sender.name.clone())))
        .thumbnail("attachment://fishingrod_smaller.png")
        .description("Your information")
        .field("Balance:", format!("${}", user_data.money), false)
        .fields(vec! [
            ("Fish caught:", format!("{}", user_data.fish_caught), true),
            ("Unique catches:", format!("{}/{}", user_data.has_seen.len(), fish_data.fish.len()), true)
        ])
        .field("Rod:",
               format!("**{}**\n- Catch Chance: {}%\n- Avg Catch Rate: ~{} seconds\n- Max Depth: {}\n- Max Weight: {}",
                       rod,
                       rod.get_catch_chance() / 10,
                       rod.get_catch_rate(),
                       rod.get_depth(), rod.get_weight_limit()), false)
        .color(Colour::TEAL)
        .timestamp(Timestamp::now());

    // create the message builder
    let builder = CreateInteractionResponse::Message(CreateInteractionResponseMessage::new()
        .embed(embed)
        .add_file(CreateAttachment::path("./assets/fishingrod_smaller.png").await.unwrap()));

    // send the message
    if let Err(err) = cmd.create_response(&ctx.http, builder).await {
        nay!("Failed to respond to command: {}", err)
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("info")
        .description("View your info")
        .dm_permission(false)
}