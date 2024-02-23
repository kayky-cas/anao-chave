use serenity::{
    all::CommandInteraction,
    builder::{CreateInteractionResponse, CreateInteractionResponseMessage},
    client::Context,
};

pub mod mine;
pub mod ping;

pub async fn default_command(
    ctx: &Context,
    command: &CommandInteraction,
) -> Result<(), serenity::prelude::SerenityError> {
    let data = CreateInteractionResponseMessage::new().content("Escreve direito filho da puta");
    let response = CreateInteractionResponse::Message(data);

    command.create_response(&ctx.http, response).await
}
