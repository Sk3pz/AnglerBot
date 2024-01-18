use serenity::all::{Colour, CommandInteraction, CommandOptionType,
                    Context, CreateAttachment, CreateCommand,
                    CreateCommandOption, CreateEmbed, CreateInteractionResponse,
                    CreateInteractionResponseMessage, ResolvedOption,
                    ResolvedValue, Timestamp};
use crate::data::rods::RodData;
use crate::nay;

pub async fn run(options: &[ResolvedOption<'_>], ctx: &Context, cmd: &CommandInteraction) {
    // get the current shop
    let shop = crate::data::shop::Shop::load();
    let rod_data = RodData::load();

    // get the index of the item to buy
    if let Some(ResolvedOption {
                    value: ResolvedValue::Integer(item_index), ..
                }) = options.first() {
        let item_index = *item_index as usize - 1;

        let rod_name = &shop.rods[item_index];
        let rod = rod_data.get_base_by_name(rod_name).unwrap();

        // create the embedded message
        let embed = CreateEmbed::new()
            .title(format!("{} Info", rod.name))
            .thumbnail("attachment://fishingrod_smaller.png")
            .description(format!("This rod is item #{} in the shop", item_index + 1))
            .field("Description:", rod.description.clone(), false)
            .field("Catch Chance:", format!("{}%", (rod.catch_chance * 100.0).round() as u32), false)
            .field("Avg Catch Rate:", format!("{}", rod.catch_rate), false)
            .field("Max Depth:", format!("{}", rod.depth), false)
            .field("Max Weight:", format!("{}", rod.weight_limit), false)
            .color(Colour::GOLD)
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
}

pub fn register() -> CreateCommand {
    CreateCommand::new("rod")
        .description("View info about a rod in the shop")
        .add_option(CreateCommandOption::new(CommandOptionType::Integer, "item",
                                              "The item to buy from the shop")
                         .min_int_value(1)
                         .max_int_value(6))
        .dm_permission(false)
}