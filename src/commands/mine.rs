use std::{
    fmt::Display,
    str::FromStr,
    sync::{Arc, Mutex},
};

use log::debug;
use serenity::{
    all::{
        AttachmentId, ButtonStyle, CommandDataOption, CommandDataOptionValue, CommandInteraction,
        CommandOptionType,
    },
    builder::{
        CreateActionRow, CreateButton, CreateCommand, CreateCommandOption, CreateEmbed,
        CreateInteractionResponse, CreateInteractionResponseMessage,
    },
    client::Context,
    model::Colour,
};

use crate::State;

pub fn register() -> CreateCommand {
    CreateCommand::new("mine")
        .description("A mine command")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "criar",
                "Coordenadas do Minecraft",
            )
            .add_sub_option(
                CreateCommandOption::new(CommandOptionType::String, "nome", "Nome do lugar")
                    .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(CommandOptionType::Integer, "x", "Coordenada X")
                    .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(CommandOptionType::Integer, "y", "Coordenada Y")
                    .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(CommandOptionType::Integer, "z", "Coordenada Z")
                    .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(CommandOptionType::String, "dimensao", "Dimensao")
                    .add_string_choice("Overworld", "overworld")
                    .add_string_choice("Nether", "nether")
                    .add_string_choice("End", "end")
                    .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(CommandOptionType::Attachment, "imagem", "Print do lugar")
                    .required(false),
            )
            .required(false),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "coordenadas",
                "Coordenadas do Minecraft",
            )
            .required(false),
        )
}

pub async fn coordinate_already_exisits(
    ctx: &Context,
    coordinate: &Coordinates,
    command: &CommandInteraction,
) -> Result<(), serenity::prelude::SerenityError> {
    let data = CreateInteractionResponseMessage::new()
        .embed(
            CreateEmbed::new()
                .title("Criar Coordenada")
                .description(format!(
                    "A coordenada {} já existe, deseja sobrescrever?",
                    coordinate.name
                ))
                .image("https://pbs.twimg.com/media/ETv8NxFXQAcQcfY?format=jpg&name=large")
                .colour(Colour::BLURPLE),
        )
        .components(vec![CreateActionRow::Buttons(vec![])]);

    let response = CreateInteractionResponse::Message(data);
    command.create_response(&ctx.http, response).await
}

pub async fn run(
    ctx: &Context,
    command: &CommandInteraction,
    state: Arc<Mutex<State>>,
) -> Result<(), serenity::prelude::SerenityError> {
    debug!("Discord: {:?}", command);

    if let Some(CommandDataOptionValue::SubCommand(commands)) =
        command.data.options.first().map(|x| &x.value)
    {
        let accept_button = CreateButton::new("accept")
            .label("Sim")
            .style(ButtonStyle::Success);

        let refuse_button = CreateButton::new("refuse")
            .label("Não")
            .style(ButtonStyle::Danger);

        let coordinate = match commands.try_into() {
            Ok(c) => c,
            Err(_) => return show_coordinates(ctx, command, state).await,
        };

        let Coordinates {
            ref name, x, y, z, ..
        } = coordinate;

        if state.lock().unwrap().database.contains_key(name) {
            return coordinate_already_exisits(ctx, &coordinate, command).await;
        }

        let action_row = CreateActionRow::Buttons(vec![refuse_button, accept_button]);

        let data = CreateInteractionResponseMessage::new()
            .embed(
                CreateEmbed::new()
                    .title("Criar Coordenada")
                    .description(format!(
                        "Voce deseja criar a coordenada\n{}:\n\tX: {}\n\tY: {}\n\t Z: {}",
                        name, x, y, z
                    ))
                    .image("https://pbs.twimg.com/media/ETv8NxFXQAcQcfY?format=jpg&name=large")
                    .colour(Colour::BLURPLE),
            )
            .components(vec![action_row]);

        let response = CreateInteractionResponse::Message(data);

        command.create_response(&ctx.http, response).await
    } else {
        show_coordinates(ctx, command, state).await
    }
}

async fn show_coordinates(
    ctx: &Context,
    command: &CommandInteraction,
    state: Arc<Mutex<State>>,
) -> Result<(), serenity::prelude::SerenityError> {
    let fields = state
        .lock()
        .unwrap()
        .database
        .values()
        .map(|v| {
            let description = format!("X: **{}** Y: **{}** Z: **{}**", v.x, v.y, v.z);

            let description = match v.image {
                Some(_image) => format!(
                    "\n[{}]({})",
                    description,
                    "https://pbs.twimg.com/media/ETv8NxFXQAcQcfY?format=jpg&name=large"
                ),
                None => description,
            };

            let description = format!("- {}\n- {}\n", description, v.dimension);

            (v.name.clone(), description, true)
        })
        .collect::<Vec<_>>();

    let data = CreateInteractionResponseMessage::new().embed(
        CreateEmbed::new()
            .title("Coordenadas: ")
            .fields(fields)
            .colour(Colour::DARK_GREEN),
    );

    let response = CreateInteractionResponse::Message(data);

    command.create_response(&ctx.http, response).await
}

pub enum Dimension {
    Overworld,
    Nether,
    End,
}

impl Display for Dimension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Dimension::Overworld => "Overworld",
            Dimension::Nether => "Nether",
            Dimension::End => "End",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for Dimension {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "overworld" => Ok(Dimension::Overworld),
            "nether" => Ok(Dimension::Nether),
            "end" => Ok(Dimension::End),
            _ => Err(()),
        }
    }
}

pub struct Coordinates {
    name: String,
    x: i64,
    y: i64,
    z: i64,
    image: Option<AttachmentId>,
    dimension: Dimension,
}

impl Coordinates {
    pub fn new(
        name: String,
        x: i64,
        y: i64,
        z: i64,
        dimension: Dimension,
        image: Option<AttachmentId>,
    ) -> Self {
        Self {
            name,
            x,
            y,
            z,
            dimension,
            image,
        }
    }
}

impl TryFrom<&Vec<CommandDataOption>> for Coordinates {
    type Error = ();

    fn try_from(value: &Vec<CommandDataOption>) -> Result<Self, Self::Error> {
        let name = value
            .iter()
            .find(|c| c.name == "nome")
            .and_then(|c| c.value.as_str())
            .ok_or(())?;
        let x = value
            .iter()
            .find(|c| c.name == "x")
            .and_then(|c| c.value.as_i64())
            .ok_or(())?;
        let y = value
            .iter()
            .find(|c| c.name == "y")
            .and_then(|c| c.value.as_i64())
            .ok_or(())?;
        let z = value
            .iter()
            .find(|c| c.name == "z")
            .and_then(|c| c.value.as_i64())
            .ok_or(())?;
        let dimension = value
            .iter()
            .find(|c| c.name == "dimensao")
            .and_then(|c| c.value.as_str())
            .and_then(|s| s.parse().ok())
            .ok_or(())?;
        let image = value
            .iter()
            .find(|c| c.name == "imagem")
            .and_then(|c| c.value.as_attachment_id());

        Ok(Coordinates::new(name.to_owned(), x, y, z, dimension, image))
    }
}
