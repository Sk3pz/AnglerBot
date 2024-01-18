use serenity::all::{Colour, CommandInteraction, Context, CreateAttachment, CreateEmbed,
                    CreateEmbedFooter, CreateInteractionResponse, CreateInteractionResponseMessage,
                    GuildId, User};
use serenity::builder::CreateCommand;
use crate::data::fish::FishData;
use crate::data::userfile::read_userfile;
use crate::nay;

pub async fn run(ctx: &Context, cmd: &CommandInteraction, sender: &User, guild: &GuildId) {
    let user_data = read_userfile(&sender.id, guild.get());

    let fish_data = FishData::load();

    let caught_fish_info = user_data.has_seen.iter().map(|name| {
        let fish = fish_data.fish_type_by_name(name).unwrap();
        (name.clone(),
         format!("Can be found between {}lbs to {}lbs below {}ft", fish.min_weight, fish.max_weight, fish.depth),
         false)
    }).collect::<Vec<(String, String, bool)>>();

    // create the embedded message
    let embed = CreateEmbed::new()
        .title(format!("{}'s Bestiary", sender.global_name.clone().unwrap_or(sender.name.clone())))
        .thumbnail("attachment://rod_with_fish.png")
        .description("Fish types you have caught:")
        .fields(caught_fish_info)
        .color(Colour::TEAL)
        .footer(CreateEmbedFooter::new(format!("{}/{}", user_data.has_seen.len(), fish_data.fish.len())));

    // create the message builder
    let builder = CreateInteractionResponse::Message(CreateInteractionResponseMessage::new()
        .embed(embed)
        .add_file(CreateAttachment::path("./assets/rod_with_fish.png").await.unwrap()));

    // send the message
    if let Err(err) = cmd.create_response(&ctx.http, builder).await {
        nay!("Failed to respond to command: {}", err)
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("bestiary")
        .description("View stats on the fish you have caught")
        .dm_permission(false)
}