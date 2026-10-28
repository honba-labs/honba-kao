//! Telegram notifications API namespace. Mirrors `web/src/lib/api/telegram.ts`.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TelegramStatus {
    pub configured: bool,
    pub chat_id: Option<&'static str>,
}

pub fn get_status() -> TelegramStatus {
    TelegramStatus {
        configured: false,
        chat_id: None,
    }
}
