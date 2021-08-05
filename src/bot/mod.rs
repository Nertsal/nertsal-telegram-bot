use super::*;

mod commands;
mod config;
mod sheets;

pub use commands::*;
use config::*;

pub struct Bot {
    config: BotConfig,
    api: Api,
    active_users: HashMap<ChatId, HashSet<String>>,
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
            active_users: HashMap::new(),
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

    pub fn get_active_users_count(&self, chat_id: &ChatId) -> usize {
        self.active_users
            .get(chat_id)
            .map(|chat| chat.len())
            .unwrap_or_default()
            + 1
    }

    pub fn get_active_users(&self, chat_id: &ChatId) -> Option<&HashSet<String>> {
        self.active_users.get(chat_id)
    }

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
        self.backup_auto();
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
        self.backup_auto();
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
            &self.active_users,
        )?;
        Ok(())
    }

    fn backup_load(&mut self, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        self.active_users =
            serde_json::from_reader(std::io::BufReader::new(std::fs::File::open(path)?))?;
        Ok(())
    }
}
