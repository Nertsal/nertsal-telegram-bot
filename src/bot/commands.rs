use super::*;
use rand::seq::IteratorRandom;

impl Bot {
    pub fn select(&mut self) -> Response {
        let chat_id = self.active_chat.unwrap();
        let user_count = async_std::task::block_on(
            self.api()
                .send(GetChatMembersCount::new(chat_id)),
        )
        .unwrap();
        let known_count = self.get_users_count(&chat_id) as i64;
        let mut response = if user_count == known_count {
            format!("/select was called succefully. ")
        } else {
            format!(
                "/select was called, but chat users info is not relevant. There are {} users in the chat, but only {} are known. ", 
                user_count, 
                known_count
            )
        };

        let mut users = self.get_active_users(&chat_id).unwrap();
        if users.len() == 0 {
            self.reset_chosen_users(chat_id);
            users = self.get_active_users(&chat_id).unwrap();
        }
        let random_user = users.iter().choose(&mut rand::thread_rng()).unwrap().clone();
        if self.choose_active_user(chat_id, random_user.clone()) {
            response.push_str(&format!("{} has been chosen!", random_user))
        }
        self.queue_save_sheets = true;
        Some(response)
    }
}

pub fn bot_commands() -> Commands<Bot> {
    Commands::new(vec![
        CommandNode::Literal {
            literals: vec!["/select".to_owned()],
            child_nodes: vec![CommandNode::Final {
                authority_level: 0,
                command: Arc::new(|bot, _, _| bot.select()),
            }],
        },
        CommandNode::Literal {
            literals: vec!["/save".to_owned()],
            child_nodes: vec![CommandNode::Final {
                authority_level: 0,
                command: Arc::new(|bot, _, _| {
                    bot.queue_save_sheets = true;
                    None
                }),
            }],
        },
    ])
}
