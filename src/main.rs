use std::env;

use axum::Router;
use log::{error, info};

use dotenv::dotenv;
use serenity::{
    all::{CurrentUser, Interaction, Message, Ready},
    async_trait,
    client::EventHandler,
    prelude::*,
};
use tokio::net::TcpListener;

struct ClientHandler;

#[async_trait]
impl EventHandler for ClientHandler {
    async fn message(&self, ctx: Context, message: Message) {
        match message.content.as_str() {
            "!ping" => {
                if let Err(why) = message.channel_id.say(&ctx.http, "Pong!").await {
                    println!("Error sending message: {:?}", why);
                }
            }
            "!hello" => {
                if let Err(why) = message.channel_id.say(&ctx.http, "Hello!").await {
                    println!("Error sending message: {:?}", why);
                }
            }
            _ => {}
        }
    }

    async fn interaction_create(&self, _: Context, interaction: Interaction) {
        info!("Interaction: {:?}", interaction);
    }

    async fn ready(&self, _: Context, ready: Ready) {
        let user_info = Self::user_display_format(&ready.user);

        info!("DISCORD: {} is up", user_info);

        for guild in ready.guilds {
            info!(
                "DISCORD: {} is connected to guild: id({})",
                user_info, guild.id
            )
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

    let mut client = Client::builder(token, GatewayIntents::default())
        .event_handler(ClientHandler)
        .await?;

    let discord_bot = tokio::spawn(async move {
        if let Err(why) = client.start().await {
            error!("DISCORD: {:?}", why);
        }
    });

    let app = Router::new();

    let listener = TcpListener::bind("0.0.0.0:3000").await?;

    axum::serve(listener, app).await?;

    discord_bot.await?;

    Ok(())
}
