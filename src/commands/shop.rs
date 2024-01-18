use serenity::all::{Colour, CommandInteraction, Context, CreateCommand, CreateEmbed, CreateEmbedFooter, CreateInteractionResponse,
                    CreateInteractionResponseMessage};
use crate::data::multipliers::MultiplierData;
use crate::data::rods::RodData;
use crate::nay;

pub async fn run(ctx: &Context, cmd: &CommandInteraction) {
    let shop = crate::data::shop::Shop::load();
    let rod_data = RodData::load();

    let multiplier = MultiplierData::load();

    let mut fields = Vec::new();

    for x in 0..shop.rods.len() {
        let item = &shop.rods[x];

        let base_rod = rod_data.get_base_by_name(item).unwrap();

        let mut cost = base_rod.cost;

        if multiplier.shop_discount != 0.0 {
            cost -= cost * multiplier.shop_discount;
        }

        fields.push((format!("{}: {}", x + 1, base_rod.name),
                     format!("${}\nRarity: {}", cost, base_rod.rarity), false));
    }

    // create the embedded message
    let embed = CreateEmbed::new()
        .title("Fishing Shop")
        .description("Run `/buy <#>` to buy an item from the shop!\nRun `/rod #` to view information about a rod\n**Today's Stock:**")
        .fields(fields)
        .footer(CreateEmbedFooter::new(format!("Next restock in: {}", shop.get_time_until_restock())))
        .color(Colour::DARK_GOLD);

    // create the message builder
    let builder = CreateInteractionResponse::Message(CreateInteractionResponseMessage::new()
        .embed(embed));

    // send the message
    if let Err(err) = cmd.create_response(&ctx.http, builder).await {
        nay!("Failed to respond to command: {}", err)
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("shop")
        .description("View today's shop")
        .dm_permission(false)
}