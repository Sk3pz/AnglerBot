use std::sync::Arc;
use std::time::Duration;
use rand::{Rng, thread_rng};
use serenity::all::{ChannelId, Colour, Context, CreateAttachment, CreateCommand, CreateEmbed,
                    CreateMessage, GuildId, Http, Mentionable, Timestamp, User, UserId};
use crate::{nay, say, wow};
use crate::data::fish::{Fish, FishData};
use crate::data::multipliers::MultiplierData;
use crate::data::userfile::{read_userfile, update_userfile, UserValues};

const WEIGHT_ADD_TIME: f32 = 0.05;

pub struct FishCatch {
    pub(crate) fish: Fish,
    pub(crate) user_file: UserValues,
    pub(crate) will_catch: bool,
    pub(crate) override_special: bool,
}

// todo: clown fish event

async fn turtle(http: Arc<Http>, channel: ChannelId, sender: UserId, guild: &GuildId, mut catch: FishCatch) {
    // create the embedded message
    let embed = CreateEmbed::new()
        .title("TURTLE EVENT")
        .thumbnail("attachment://turtle.png")
        .description(format!("A turtle stole your **{}**!\n:turtle::turtle::turtle:", catch.fish))
        .color(Colour::DARK_GREEN)
        .timestamp(Timestamp::now());

    // create the message builder
    let builder = CreateMessage::new()
        .content(format!("{}", sender.mention()))
        .embed(embed)
        .add_file(CreateAttachment::path("./assets/turtle.png").await.unwrap());

    // send the message
    let msg = channel.send_message(&http, builder).await;
    if let Err(e) = msg {
        nay!("Failed to send message: {}", e);
    }

    catch.user_file.cast = false;
    update_userfile(&sender, catch.user_file, guild.get());
}

pub async fn catch(http: Arc<Http>, channel: ChannelId, sender: UserId, guild_id: GuildId, mut catch: FishCatch) {
    let value = catch.fish.get_value().max(1);
    let rod = catch.user_file.get_rod().clone();

    // if the fish is too heavy for the rod, break the rod
    let weight_limit = rod.get_weight_limit();
    if catch.fish.weight > weight_limit as f32 && !catch.override_special {
        let msg =
            channel.send_message(&http,
                                 CreateMessage::new().content(format!("{} Your line broke! The {}lb **{}** was too heavy!",
                                                                      sender.mention(), catch.fish.weight, catch.fish))).await;
        if let Err(e) = msg {
            nay!("Failed to send message: {}", e);
        }
        catch.user_file.cast = false;
        update_userfile(&sender, catch.user_file, guild_id.get());
        return;
    }

    if !catch.will_catch && !catch.override_special {
        let msg =
            channel.send_message(&http,
                                 CreateMessage::new().content(format!("{} A {}lb **{}** got away! Better luck next time!",
                                                                      sender.mention(), catch.fish.weight, catch.fish))).await;
        if let Err(e) = msg {
            nay!("Failed to send message: {}", e);
        }
        catch.user_file.cast = false;
        update_userfile(&sender, catch.user_file, guild_id.get());
        return;
    }

    let turtle_chance = thread_rng().gen_range(0..100) >= 98;
    if turtle_chance && !catch.override_special {
        turtle(http, channel, sender, &guild_id, catch).await;
        return;
    }

    let fish_data = FishData::load();

    if !catch.user_file.has_seen.contains(&catch.fish.fish_type.name) {
        catch.user_file.has_seen.push(catch.fish.fish_type.name.clone());
    }
    catch.user_file.money += value;
    catch.user_file.fish_caught += 1;

    // create the embedded message
    let embed = CreateEmbed::new()
        .title("You caught a fish!")
        .thumbnail("attachment://rod_with_fish.png")
        .description(format!("You caught a **{}** at {}lbs!", catch.fish, catch.fish.weight))
        .fields(vec![
            ("Value:", format!("${}", value), true),
            ("New Balance:", format!("${}", catch.user_file.money), true),
        ])
        .field("Your Rod:", format!("{}", rod), false)
        .fields(vec! [
        ("Fish caught:", format!("{}", catch.user_file.fish_caught), true),
        ("Unique catches:", format!("{}/{}", catch.user_file.has_seen.len(), fish_data.fish.len()), true)
    ])
        .color(Colour::DARK_TEAL)
        .timestamp(Timestamp::now());

    // create the message builder
    let builder = CreateMessage::new()
        .content(format!("{} has caught a fish!", sender.mention()))
        .embed(embed)
        .add_file(CreateAttachment::path("./assets/rod_with_fish.png").await.unwrap());

    // send the message
    let msg = channel.send_message(&http, builder).await;
    if let Err(e) = msg {
        nay!("Failed to send message: {}", e);
    }

    // update the user file
    catch.user_file.cast = false;
    update_userfile(&sender, catch.user_file, guild_id.get());
}

pub fn run(ctx: &Context, channel: &ChannelId, sender: &User, guild: &GuildId) -> String {
    let mut user_file = read_userfile(&sender.id, guild.get());
    if user_file.cast {
        return "You have already cast your line!".to_string();
    }
    user_file.cast = true;
    update_userfile(&sender.id, user_file, guild.get());
    user_file = read_userfile(&sender.id, guild.get());

    let fish_data = FishData::load();

    let rod = &user_file.get_rod();

    let fish = Fish::random_fish(&fish_data, rod);

    let multipliers = MultiplierData::load();

    let will_catch = thread_rng().gen_range(0..1000) <= (rod.get_catch_chance() + multipliers.catch_chance);

    // get the time until catch and convert to miliseconds
    let weight_catch_time_add = (fish.weight - fish.fish_type.avg_weight as f32) * WEIGHT_ADD_TIME;
    let catch_time = ((rod.random_catch_time() + weight_catch_time_add) * 1000.0) as u64;

    let http = ctx.http.clone();
    let channel_id = *channel;
    let fish_clone= fish.clone();
    let id = sender.id;
    let guild_id = *guild;
    let fish_weight = fish.weight;
    let ufile = user_file.clone();

    // schedule the catch
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(catch_time)).await;
        catch(http, channel_id, id, guild_id, FishCatch {user_file, fish, will_catch, override_special: false}).await;
    });

    if fish_clone.rarity.ident() > 3 {
        wow!("{} {} catch a {} of {}lbs in {} seconds! Value: {}", sender.name,
        if will_catch { "will" } else { "wont" }, fish_clone, fish_weight, catch_time / 1000u64, fish_clone.get_value());
    } else {
        say!("{} {} catch a {} of {}lbs in {} seconds! Value: {}", sender.name,
        if will_catch { "will" } else { "wont" }, fish_clone, fish_weight, catch_time / 1000u64, fish_clone.get_value());
    }

    format!("You have cast your {}.", ufile.get_rod())
}

pub fn register(motd: String) -> CreateCommand {
    CreateCommand::new("fish")
        .description(motd)
        .dm_permission(false)
}