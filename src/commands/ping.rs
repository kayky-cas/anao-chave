use log::debug;
use serenity::{
    all::CommandInteraction,
    builder::{CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage},
    client::Context,
};

pub fn register() -> CreateCommand {
    CreateCommand::new("ping").description("A ping command")
}

pub async fn run(
    ctx: &Context,
    command: &CommandInteraction,
) -> Result<(), serenity::prelude::SerenityError> {
    debug!("Discord: {:?}", command);

    let data = CreateInteractionResponseMessage::new().content("Pong!");
    let response = CreateInteractionResponse::Message(data);

    command.create_response(&ctx.http, response).await
}
