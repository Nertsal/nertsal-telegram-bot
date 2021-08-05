use super::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct UsersState {
    active_users: HashMap<ChatId, HashSet<String>>,
    chosen_users: HashMap<ChatId, HashSet<String>>,
}

impl UsersState {
    pub fn new() -> Self {
        Self {
            active_users: HashMap::new(),
            chosen_users: HashMap::new(),
        }
    }

    pub fn get_active_users_count(&self, chat_id: &ChatId) -> usize {
        self.active_users
            .get(chat_id)
            .map(|chat| chat.len())
            .unwrap_or_default()
    }

    pub fn get_chosen_users_count(&self, chat_id: &ChatId) -> usize {
        self.chosen_users
            .get(chat_id)
            .map(|chat| chat.len())
            .unwrap_or_default()
    }

    pub fn get_active_users(&self, chat_id: &ChatId) -> Option<&HashSet<String>> {
        self.active_users.get(chat_id)
    }

    pub fn get_chosen_users(&self, chat_id: &ChatId) -> Option<&HashSet<String>> {
        self.chosen_users.get(chat_id)
    }

    pub fn add_active_user(&mut self, chat_id: ChatId, user_name: String) -> bool {
        !self
            .chosen_users
            .entry(chat_id)
            .or_default()
            .contains(&user_name)
            && self
                .active_users
                .entry(chat_id)
                .or_default()
                .insert(user_name)
    }

    pub fn remove_active_user(&mut self, chat_id: ChatId, user_name: &String) -> bool {
        self.active_users
            .get_mut(&chat_id)
            .map(|chat| chat.remove(user_name))
            .unwrap_or(false)
    }

    pub fn add_chosen_user(&mut self, chat_id: ChatId, user_name: String) {
        assert!(self
            .chosen_users
            .entry(chat_id)
            .or_default()
            .insert(user_name))
    }

    pub fn reset_chosen_users(&mut self, chat_id: ChatId) {
        let active_users = self.active_users.entry(chat_id).or_default();
        for chosen_user in self.chosen_users.remove(&chat_id).unwrap_or_default() {
            active_users.insert(chosen_user);
        }
    }
}
