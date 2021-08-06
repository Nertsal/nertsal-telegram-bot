use super::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct UsersState {
    pub active_users: HashSet<ChatUser>,
    pub chosen_users: HashSet<ChatUser>,
}

impl UsersState {
    pub fn new() -> Self {
        Self {
            active_users: HashSet::new(),
            chosen_users: HashSet::new(),
        }
    }

    pub fn add_active_user(&mut self, user: ChatUser) -> bool {
        !self.chosen_users.contains(&user) && self.active_users.insert(user)
    }

    pub fn remove_active_user(&mut self, user: &ChatUser) -> bool {
        self.active_users.remove(user)
    }

    pub fn add_chosen_user(&mut self, user: ChatUser) {
        assert!(self.chosen_users.insert(user))
    }

    pub fn reset_chosen_users(&mut self) {
        for chosen_user in self.chosen_users.drain() {
            self.active_users.insert(chosen_user);
        }
    }
}

#[derive(Clone, Hash, Eq, Serialize, Deserialize)]
pub struct ChatUser {
    pub name: String,
    pub id: UserId,
}

impl ChatUser {
    pub fn new(user: &User) -> Self {
        Self {
            name: get_user_name(user),
            id: user.id,
        }
    }
}

impl PartialEq for ChatUser {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
