use futures::StreamExt;
use nertsal_commands::*;
use std::{
    collections::{HashMap, HashSet},
    io::Read,
    sync::Arc,
};
use telegram_bot::*;
use tokio_compat_02::FutureExt;

mod bot;

use bot::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut buffer = String::new();
    std::io::BufReader::new(std::fs::File::open("secrets/token.txt").unwrap())
        .read_to_string(&mut buffer)
        .unwrap();
    let token = buffer;
    let api = Api::new(token);

    let config = serde_json::from_reader(std::io::BufReader::new(
        std::fs::File::open("config/bot_config.json").unwrap(),
    ))
    .unwrap();

    // Initialize bot and commands
    let mut bot = Bot::new(config, &api);
    bot.setup_google_sheets();
    let commands = bot_commands();

    // Fetch new updates via long poll method
    let mut stream = api.stream();
    while let Some(update) = stream.next().compat().await {
        match update {
            Ok(update) => match update.kind {
                UpdateKind::Message(message) => match message.kind {
                    MessageKind::Text { ref data, .. } => {
                        bot.check_active_user(message.chat.id(), &message.from);
                        println!(
                            "[{}] {}: {}",
                            message.chat.id(),
                            get_user_name(&message.from),
                            data
                        );
                        bot.active_chat = Some(message.chat.id());
                        let command_message = CommandMessage {
                            sender_name: get_user_name(&message.from),
                            authority_level: 0,
                            message_text: data.to_owned(),
                        };
                        for response in commands.perform_commands(&mut bot, &command_message) {
                            if let Some(response) = response {
                                api.send(message.text_reply(response)).compat().await?;
                            }
                        }
                    }
                    MessageKind::NewChatMembers { data } => {
                        for user in data {
                            bot.add_active_user(message.chat.id(), &user);
                        }
                    }
                    MessageKind::LeftChatMember { data } => {
                        bot.remove_active_user(message.chat.id(), &data);
                    }
                    _ => println!("Unhandled message: {:?}", message),
                },
                _ => println!("Unhandled update: {:?}", update),
            },
            Err(err) => println!("An error occured: {:?}", err),
        }

        bot.update().compat().await;
    }
    Ok(())
}

fn get_user_name(user: &User) -> String {
    user.username
        .clone()
        .unwrap_or_else(|| user.first_name.clone())
}
