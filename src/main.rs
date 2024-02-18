use log::{error, info};
use std::env;

use dotenv::dotenv;
use serenity::{
    all::{CurrentUser, Interaction, Ready},
    async_trait,
    builder::CreateCommand,
    client::EventHandler,
    prelude::*,
};

struct ClientHandler;

#[async_trait]
impl EventHandler for ClientHandler {
    async fn interaction_create(&self, _: Context, interaction: Interaction) {
        info!("Interaction: {:?}", interaction);
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        let user_info = Self::user_display_format(&ready.user);

        info!("DISCORD: {} is up", user_info);

        for guild in ready.guilds {
            info!(
                "DISCORD: {} is connected to guild: id({})",
                user_info, guild.id
            );

            let _ = guild
                .id
                .set_commands(
                    &ctx.http,
                    vec![CreateCommand::new("ping").description("A ping command")],
                )
                .await;
        }
    }
}

impl ClientHandler {
    fn user_display_format(user: &CurrentUser) -> String {
        format!("{}({})", user.name, user.id)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    env_logger::init();

    info!("Starting up");

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let intents = GatewayIntents::GUILDS
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_MESSAGE_REACTIONS;

    let mut client = Client::builder(token, intents)
        .event_handler(ClientHandler)
        .await?;

    info!("DISCORD: Starting client");
    if let Err(why) = client.start().await {
        error!("DISCORD: {:?}", why);
    }

    Ok(())
}
