use futures::StreamExt;
use nertsal_commands::*;
use std::{
    collections::{HashMap, HashSet},
    io::Read,
    sync::Arc,
};
use telegram_bot::*;

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

    // Initialize bot and commands
    let mut bot = Bot::new(&api);
    let commands = Commands::<Bot>::new(vec![CommandNode::Literal {
        literals: vec!["/select".to_owned()],
        child_nodes: vec![CommandNode::Final {
            authority_level: 0,
            command: Arc::new(|bot, _, _| {
                let user_count = async_std::task::block_on(
                    bot.api()
                        .send(GetChatMembersCount::new(&bot.active_chat.unwrap())),
                )
                .unwrap();
                let known_count = bot.get_active_users_count(&bot.active_chat.unwrap()) as i64;
                if user_count == known_count {
                    Some(format!("/select was called succefully"))
                } else {
                    Some(format!(
                        "/select was called, but chat users info is not relevant. There are {} users in the chat, but only {} are known", 
                        user_count, 
                        known_count
                    ))
                }
            }),
        }],
    }]);

    // Fetch new updates via long poll method
    let mut stream = api.stream();
    while let Some(update) = stream.next().await {
        match update {
            Ok(update) => match update.kind {
                UpdateKind::Message(message) => match message.kind {
                    MessageKind::Text { ref data, .. } => {
                        bot.check_active_user(message.chat.id(), &message.from);
                        bot.active_chat = Some(message.chat.id());
                        let command_message = CommandMessage {
                            sender_name: get_user_name(&message.from),
                            authority_level: 0,
                            message_text: data.to_owned(),
                        };
                        for response in commands.perform_commands(&mut bot, &command_message) {
                            if let Some(response) = response {
                                api.send(message.text_reply(response)).await?;
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
    }
    Ok(())
}

fn get_user_name(user: &User) -> String {
    user.username
        .clone()
        .unwrap_or_else(|| user.first_name.clone())
}
