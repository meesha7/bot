use crate::{commands::*, db::setup_db, listeners::Handler, prelude::*, utils::framework::*};
use serenity::{
    client::bridge::gateway::GatewayIntents, framework::StandardFramework, http::Http, model::prelude::*, prelude::*,
};
use std::{
    collections::{HashMap, HashSet},
    env,
    sync::Arc,
};

pub mod commands;
pub mod data;
pub mod db;
pub mod listeners;
pub mod prelude;
pub mod utils;

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to initialize dotenv.");
    env_logger::init();

    let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name("config"))
        .expect("Failed to open the config file.");

    setup_db().await.expect("Failed to set up database.");

    //If a token exists in the dotenv, prefer to use that.
    let token;
    if let Ok(x) = env::var("DISCORD_TOKEN") {
        token = x;
    } else {
        token = settings
            .get_str("discord_token")
            .expect("discord_token not found in config.");
    }

    let http = Http::new_with_token(&token);

    //Get the application info to use for later.
    let (owners, botid, ownerid) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id, info.owner.id)
        }
        Err(why) => panic!("Couldn't get application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| c.owners(owners).on_mention(Some(botid)).dynamic_prefix(dynamic_prefix))
        .on_dispatch_error(dispatch_error)
        .after(after)
        .normal_message(log_dm)
        .help(&HELP)
        .group(&ADMIN_GROUP)
        .group(&GAMBLING_GROUP)
        .group(&ROLES_GROUP)
        .group(&SETTINGS_GROUP)
        .group(&UTILITY_GROUP);

    let mut client = Client::builder(&token)
        .intents(GatewayIntents::all())
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating the client.");

    //Set the cache for each channel to 10000 messages.
    client.cache_and_http.cache.set_max_messages(10000).await;

    //Collect the bot owners.
    let mut bot_owners: Vec<UserId> = settings
        .get_array("bot_owners")
        .expect("bot_owners not found in config.")
        .into_iter()
        .map(|x| UserId(x.try_into::<u64>().expect("Failed to decode owner ID into UserId.")))
        .collect();
    bot_owners.push(ownerid);

    //Fill the data with previously gathered and default values.
    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<BotId>(botid);
        data.insert::<BotOwners>(bot_owners);
        data.insert::<DefaultPrefix>(
            settings
                .get_str("default_prefix")
                .expect("default_prefix not found in config."),
        );
        let map = HashMap::new();
        data.insert::<GuildPrefixes>(map);
    }

    client.start_autosharded().await.expect("Failed to start the client.");
}
