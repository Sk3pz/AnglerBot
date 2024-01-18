use serenity::all::{CreateCommand, GuildId, UserId};
use crate::data::userfile::read_userfile;

pub fn run(sender: &UserId, guild: &GuildId) -> String {
    let user_data = read_userfile(sender, guild.get());
    format!("You have ${}", user_data.money)
}

pub fn register() -> CreateCommand {
    CreateCommand::new("balance")
        .description("Check your balance")
        .dm_permission(false)
}