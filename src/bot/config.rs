use super::*;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct BotConfig {
    pub main_chat: ChatId,
    pub google_sheet_config: Option<GoogleSheetConfig>,
}

#[derive(Deserialize)]
pub struct GoogleSheetConfig {
    pub sheet_id: String,
}
