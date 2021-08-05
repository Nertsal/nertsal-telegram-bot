use super::*;

pub fn bot_commands() -> Commands<Bot> {
    Commands::new(vec![CommandNode::Literal {
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
    },
    CommandNode::Literal {
        literals: vec!["/save".to_owned()],
        child_nodes: vec![CommandNode::Final {
            authority_level: 0,
            command: Arc::new(|bot, _, _| {
                bot.queue_save_sheets = true;
                None
            })
        }]
    }])
}