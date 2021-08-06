use super::*;

mod commands;
mod config;
mod sheets;
pub mod users_state;

pub use commands::*;
use config::*;
use users_state::*;

pub struct Bot {
    pub config: BotConfig,
    api: Api,
    users_state: UsersState,
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
            self.save_to_google_sheets().await.unwrap();
        }
    }

    pub fn get_user_authority_level(&self, user: &ChatUser) -> UserAuthorityLevel {
        if self.get_all_users().any(|chat_user| *chat_user == *user) {
            let admins = async_std::task::block_on(
                self.api
                    .send(GetChatAdministrators::new(self.config.main_chat)),
            )
            .unwrap();
            if admins
                .iter()
                .any(|member| ChatUser::new(&member.user) == *user)
            {
                UserAuthorityLevel::Admin
            } else {
                UserAuthorityLevel::MainChatUser
            }
        } else {
            UserAuthorityLevel::RandomUser
        }
    }

    pub fn get_users_count(&self) -> usize {
        self.users_state.active_users.len() + self.users_state.chosen_users.len() + 1
    }

    pub fn get_all_users(&self) -> impl Iterator<Item = &ChatUser> {
        self.get_chosen_users()
            .iter()
            .chain(self.get_active_users().iter())
    }

    pub fn get_active_users(&self) -> &HashSet<ChatUser> {
        &self.users_state.active_users
    }

    pub fn get_chosen_users(&self) -> &HashSet<ChatUser> {
        &self.users_state.chosen_users
    }

    pub fn add_active_user(&mut self, user: ChatUser) {
        if self.users_state.add_active_user(user.clone()) {
            println!("User {} joined the chat", user.name);
            self.backup_auto();
        } else {
            println!("User {} joined the chat but already existed", user.name);
        }
    }

    pub fn check_active_user(&mut self, user: ChatUser) {
        if self.users_state.add_active_user(user.clone()) {
            println!("Got a message from unknown user {}", user.name);
            self.backup_auto();
        } else {
            println!("Got a message from {}", user.name);
        }
    }

    pub fn remove_active_user(&mut self, user: &ChatUser) {
        if self.users_state.remove_active_user(user) {
            println!("User {} left the chat", user.name);
            self.backup_auto();
        } else {
            println!("Unknown user {} left the chat", user.name);
        }
    }

    pub fn choose_active_user(&mut self, user: ChatUser) -> bool {
        if self.users_state.remove_active_user(&user) {
            self.users_state.add_chosen_user(user.clone());
            println!("Chose user {}", user.name);
            self.backup_auto();
            true
        } else {
            println!("Failed to choose user {}", user.name);
            false
        }
    }

    pub fn reset_chosen_users(&mut self) {
        self.users_state.reset_chosen_users();
        self.backup_auto();
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
