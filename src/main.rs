use std::env;

use dotenv::dotenv;
use serenity::{all::Message, async_trait, client::EventHandler, prelude::*};

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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let mut client = Client::builder(token, GatewayIntents::default())
        .event_handler(ClientHandler)
        .await?;

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }

    Ok(())
}
