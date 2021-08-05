use super::*;

pub struct Bot {
    api: Api,
    active_users: HashMap<ChatId, HashSet<String>>,
    pub active_chat: Option<ChatId>,
}

impl Bot {
    pub fn new(api: &Api) -> Self {
        Self {
            api: api.clone(),
            active_users: HashMap::new(),
            active_chat: None,
        }
    }

    pub fn api(&self) -> &Api {
        &self.api
    }

    pub fn get_active_users_count(&self, chat_id: &ChatId) -> usize {
        self.active_users
            .get(chat_id)
            .map(|chat| chat.len())
            .unwrap_or_default()
            + 1
    }

    // pub fn get_active_users(&self, chat_id: &ChatId) -> Option<&HashSet<String>> {
    //     self.active_users.get(chat_id)
    // }

    pub fn add_active_user(&mut self, chat_id: ChatId, user: &User) {
        let user_name = get_user_name(user);
        if self
            .active_users
            .entry(chat_id)
            .or_insert(HashSet::new())
            .insert(user_name.clone())
        {
            println!("User {} joined chat {}", user_name, chat_id);
        } else {
            println!(
                "User {} joined chat {} but already existed",
                user_name, chat_id
            );
        }
    }

    pub fn check_active_user(&mut self, chat_id: ChatId, user: &User) {
        let user_name = get_user_name(user);
        if self
            .active_users
            .entry(chat_id)
            .or_insert(HashSet::new())
            .insert(user_name.clone())
        {
            println!(
                "Got a message from unknown user {} in chat {}",
                user_name, chat_id
            );
        } else {
            println!("Got a message from {} in chat {}", user_name, chat_id);
        }
    }

    pub fn remove_active_user(&mut self, chat_id: ChatId, user: &User) {
        let user_name = get_user_name(user);
        if self
            .active_users
            .get_mut(&chat_id)
            .map(|chat| chat.remove(&user_name))
            .unwrap_or(false)
        {
            println!("User {} left chat {}", user_name, chat_id);
        } else {
            println!("Unknown user {} left chat {}", user_name, chat_id);
        }
    }
}
