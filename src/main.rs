use std::{env, sync::Arc};

use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use log::{error, info};

use dotenv::dotenv;
use serenity::{
    all::{CommandInteraction, CommandOptionType, CurrentUser, Interaction, Message, Ready},
    async_trait,
    builder::{CreateCommand, CreateCommandOption, CreateInteractionResponse},
    client::EventHandler,
    interactions_endpoint::Verifier,
    prelude::*,
};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

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
                    vec![CreateCommand::new("ping")
                        .description("Responde com pong")
                        .add_option(
                            CreateCommandOption::new(CommandOptionType::String, "", "none")
                                .required(true),
                        )],
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

struct AnaoChaveState {
    verifier: Verifier,
}

fn handle_command(_command: CommandInteraction) -> CreateInteractionResponse {
    todo!()
}

async fn handle_interaction(
    headers: HeaderMap,
    State(state): State<Arc<AnaoChaveState>>,
    body: Bytes,
) -> impl IntoResponse {
    let signature = headers
        .get("X-Signature-Ed25519")
        .and_then(|t| t.to_str().ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let timestamp = headers
        .get("X-Signature-Timestamp")
        .and_then(|t| t.to_str().ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    if state.verifier.verify(signature, timestamp, &body).is_err() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let response = match serde_json::from_slice(&body).or(Err(StatusCode::UNPROCESSABLE_ENTITY))? {
        Interaction::Ping(_) => CreateInteractionResponse::Pong,
        Interaction::Command(command) => handle_command(command),
        Interaction::Autocomplete(_) => todo!(),
        Interaction::Component(_) => todo!(),
        Interaction::Modal(_) => todo!(),
        _ => todo!(),
    };

    Ok(Json(response))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    env_logger::init();

    info!("Starting up");

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let public_key =
        env::var("DISCORD_PUBLIC_KEY").expect("Expected a public key in the environment");

    let intents = GatewayIntents::GUILDS
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_MESSAGE_REACTIONS;

    let mut client = Client::builder(token, intents)
        .event_handler(ClientHandler)
        .await?;

    let discord_bot = tokio::spawn(async move {
        info!("DISCORD: Starting client");
        if let Err(why) = client.start().await {
            error!("DISCORD: {:?}", why);
        }
    });

    let state = Arc::new(AnaoChaveState {
        verifier: Verifier::new(&public_key),
    });

    let app = Router::new()
        .route("/interactions", post(handle_interaction))
        .with_state(state)
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    let listener = TcpListener::bind("127.0.0.1:3000").await?;

    let server = tokio::spawn(async move {
        info!("HTTP: Listening on");
        if let Err(err) = axum::serve(listener, app).await {
            error!("HTTP: {:?}", err);
        }
    });

    let _ = tokio::join!(server, discord_bot);

    Ok(())
}
