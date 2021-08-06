use super::*;
use rand::seq::IteratorRandom;

impl Bot {
    fn check(&mut self) -> Response {
        let chat_id = self.config.main_chat;
        let user_count =
            async_std::task::block_on(self.api().send(GetChatMembersCount::new(chat_id))).unwrap();
        let known_count = self.get_users_count() as i64;
        Some(format!(
            "{} out of {} users are known",
            known_count, user_count,
        ))
    }

    fn select(&mut self) -> Response {
        let mut users = self.get_active_users();
        if users.len() == 0 {
            self.reset_chosen_users();
            users = self.get_active_users();
        }
        let random_user = users
            .iter()
            .choose(&mut rand::thread_rng())
            .unwrap()
            .clone();
        self.queue_save_sheets = true;
        if self.choose_active_user(random_user.clone()) {
            Some(format!("{} has been chosen!", random_user.name))
        } else {
            None
        }
    }
}

pub fn bot_commands() -> Commands<Bot, ChatUser> {
    Commands::new(vec![
        CommandNode::Literal {
            literals: vec!["/check".to_owned()],
            child_nodes: vec![CommandNode::Final {
                authority_level: UserAuthorityLevel::RandomUser as usize,
                command: Arc::new(|bot, _, _| bot.check()),
            }],
        },
        CommandNode::Literal {
            literals: vec!["/select".to_owned()],
            child_nodes: vec![CommandNode::Final {
                authority_level: UserAuthorityLevel::MainChatUser as usize,
                command: Arc::new(|bot, _, _| bot.select()),
            }],
        },
        CommandNode::Literal {
            literals: vec!["/save".to_owned()],
            child_nodes: vec![CommandNode::Final {
                authority_level: UserAuthorityLevel::Admin as usize,
                command: Arc::new(|bot, _, _| {
                    bot.queue_save_sheets = true;
                    None
                }),
            }],
        },
        CommandNode::Literal {
            literals: vec!["/authority".to_owned()],
            child_nodes: vec![CommandNode::Final {
                authority_level: UserAuthorityLevel::RandomUser as usize,
                command: Arc::new(|bot, sender, _| {
                    Some(format!(
                        "@{}, your authority level is {:?}",
                        sender.name,
                        bot.get_user_authority_level(&sender)
                    ))
                }),
            }],
        },
    ])
}

#[derive(Debug)]
pub enum UserAuthorityLevel {
    RandomUser = 0,
    MainChatUser = 1,
    Admin = 2,
}
