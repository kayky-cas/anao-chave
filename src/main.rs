mod commands;

use commands::mine::{Coordinates, Dimension};
use log::{debug, error, info};
use std::{
    collections::HashMap,
    env,
    sync::{Arc, Mutex},
};

use dotenv::dotenv;
use serenity::{
    all::{AttachmentId, CurrentUser, GatewayIntents, Interaction, Ready},
    async_trait,
    client::{Context, EventHandler},
    Client,
};

#[derive(Default)]
struct State {
    database: HashMap<String, Coordinates>,
}

impl State {
    pub fn new() -> Self {
        let database = HashMap::from([
            (
                "spawn".to_string(),
                Coordinates::new(
                    "Spawn".to_owned(),
                    0,
                    0,
                    0,
                    Dimension::Overworld,
                    Some(AttachmentId::new(1)),
                ),
            ),
            (
                "nether".to_string(),
                Coordinates::new("Nether".to_owned(), 0, 0, 0, Dimension::Nether, None),
            ),
            (
                "end".to_string(),
                Coordinates::new("End".to_owned(), 0, 0, 0, Dimension::End, None),
            ),
        ]);

        Self { database }
    }
}

#[derive(Default)]
struct ClientHandler {
    state: Arc<Mutex<State>>,
}

impl ClientHandler {
    pub fn new() -> Self {
        Self {
            //TODO: state: Arc::new(Mutex::new(State::default())),
            state: Arc::new(Mutex::new(State::new())),
        }
    }
}

#[async_trait]
impl EventHandler for ClientHandler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        debug!("Interaction: {:?}", interaction);

        if let Interaction::Command(command) = interaction {
            let command_name = command.data.name.as_str();
            match command_name {
                "ping" => {
                    commands::ping::run(&ctx, &command).await.unwrap();
                }
                "mine" => {
                    commands::mine::run(&ctx, &command, self.state.clone())
                        .await
                        .unwrap();
                }
                _ => {
                    commands::default_command(&ctx, &command).await.unwrap();
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        let user_info = Self::user_display_format(&ready.user);

        info!("DISCORD: {} is up", user_info);

        for guild in ready.guilds {
            info!(
                "DISCORD: {} is connected to guild: id({})",
                user_info, guild.id
            );

            let commands = guild
                .id
                .set_commands(
                    &ctx.http,
                    vec![commands::ping::register(), commands::mine::register()],
                )
                .await;

            debug!("DISCORD: {:?}", commands)
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
        .event_handler(ClientHandler::new())
        .await?;

    info!("DISCORD: Starting client");
    if let Err(why) = client.start().await {
        error!("DISCORD: {:?}", why);
    }

    Ok(())
}
