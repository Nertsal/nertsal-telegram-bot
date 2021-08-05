use super::*;

mod commands;
mod config;
mod sheets;
mod users_state;

pub use commands::*;
use config::*;
use users_state::*;

pub struct Bot {
    config: BotConfig,
    api: Api,
    users_state: UsersState,
    pub active_chat: Option<ChatId>,
    hub: Option<google_sheets4::Sheets>,
    queue_save_sheets: bool,
}

impl Bot {
    pub fn new(api: &Api) -> Self {
        let config = serde_json::from_reader(std::io::BufReader::new(
            std::fs::File::open("config/bot_config.json").unwrap(),
        ))
        .unwrap();
        Self {
            config,
            api: api.clone(),
            users_state: UsersState::new(),
            active_chat: None,
            hub: None,
            queue_save_sheets: false,
        }
    }

    pub fn from_backup(api: &Api) -> std::io::Result<Self> {
        let mut bot = Self::new(api);
        bot.backup_load("backups/auto")?;
        Ok(bot)
    }

    pub fn api(&self) -> &Api {
        &self.api
    }

    pub async fn update(&mut self) {
        if self.queue_save_sheets {
            self.queue_save_sheets = false;
            if let Some(active_chat) = &self.active_chat {
                self.save_to_google_sheets(active_chat).await.unwrap();
            }
        }
    }

    pub fn get_users_count(&self, chat_id: &ChatId) -> usize {
        self.users_state.get_active_users_count(chat_id)
            + self.users_state.get_chosen_users_count(chat_id)
            + 1
    }

    pub fn get_active_users(&self, chat_id: &ChatId) -> Option<&HashSet<String>> {
        self.users_state.get_active_users(chat_id)
    }

    pub fn get_chosen_users(&self, chat_id: &ChatId) -> Option<&HashSet<String>> {
        self.users_state.get_chosen_users(chat_id)
    }

    pub fn add_active_user(&mut self, chat_id: ChatId, user_name: String) {
        if self.users_state.add_active_user(chat_id, user_name.clone()) {
            println!("User {} joined chat {}", user_name, chat_id);
        } else {
            println!(
                "User {} joined chat {} but already existed",
                user_name, chat_id
            );
        }
        self.backup_auto();
    }

    pub fn check_active_user(&mut self, chat_id: ChatId, user_name: String) {
        if self
            .users_state
            .add_active_user(chat_id, user_name.clone())
        {
            println!(
                "Got a message from unknown user {} in chat {}",
                user_name, chat_id
            );
        } else {
            println!("Got a message from {} in chat {}", user_name, chat_id);
        }
        self.backup_auto();
    }

    pub fn remove_active_user(&mut self, chat_id: ChatId, user_name: &String) {
        if self.users_state.remove_active_user(chat_id, user_name) {
            println!("User {} left chat {}", user_name, chat_id);
        } else {
            println!("Unknown user {} left chat {}", user_name, chat_id);
        }
        self.backup_auto();
    }

    pub fn choose_active_user(&mut self, chat_id: ChatId, user_name: String) -> bool {
        if self.users_state.remove_active_user(chat_id, &user_name) {
            self.users_state.add_chosen_user(chat_id, user_name.clone());
            println!("Chose user {} in chat {}", user_name, chat_id);
            true
        } else {
            println!("Unable to choose user {} in chat {}", user_name, chat_id);
            false
        }
    }

    pub fn reset_chosen_users(&mut self, chat_id: ChatId) {
        self.users_state.reset_chosen_users(chat_id);
    }

    fn backup_auto(&self) {
        match self.backup_create("backups/auto") {
            Ok(_) => (),
            Err(error) => println!("Error creating backup: {}", error),
        }
    }

    fn backup_create(&self, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        serde_json::to_writer(
            std::io::BufWriter::new(std::fs::File::create(path)?),
            &self.users_state,
        )?;
        Ok(())
    }

    fn backup_load(&mut self, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        self.users_state =
            serde_json::from_reader(std::io::BufReader::new(std::fs::File::open(path)?))?;
        Ok(())
    }
}
