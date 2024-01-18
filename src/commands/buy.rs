use serenity::all::{CommandOptionType, CreateCommand, CreateCommandOption, GuildId, ResolvedOption, ResolvedValue, User};

pub fn run(options: &[ResolvedOption], sender: &User, guild: &GuildId) -> String {
    // get the current shop
    let mut shop = crate::data::shop::Shop::load();

    let user_file = crate::data::userfile::read_userfile(&sender.id, guild.get());

    if user_file.cast {
        return "Please wait until your cast is finished to buy a new rod!".to_string();
    }

    // get the index of the item to buy
    if let Some(ResolvedOption {
                value: ResolvedValue::Integer(item_index), ..
                }) = options.first() {
        let item_index = *item_index as usize - 1;

        let buy_result = shop.sell_rod(item_index, guild.get(), &sender.id);

        if let Err(e) = buy_result {
            return match e {
                crate::data::shop::BuyError::InvalidRod => "Invalid item!".to_string(),
                crate::data::shop::BuyError::NoMoney => "You don't have enough money!".to_string(),
            };
        }

        buy_result.unwrap_or("Invalid item!".to_string())
    } else {
        "Invalid item!".to_string()
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("buy")
        .description("Buy an item from the shop")
        .dm_permission(false)
        .add_option(CreateCommandOption::new(CommandOptionType::Integer, "item",
                                             "The item to buy from the shop")
            .min_int_value(1)
            .max_int_value(6)
            .required(true))
}