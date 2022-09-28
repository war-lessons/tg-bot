use crate::{Lang, Translate, CONF};
use teloxide::{
    adaptors::AutoSend,
    payloads::SendMessageSetters,
    requests::{Requester, ResponseResult},
    types::{CallbackQuery, ChatId, Message, ParseMode},
    Bot,
};

pub type Reply = <AutoSend<Bot> as Requester>::SendMessage;
pub type ReplyResult = ResponseResult<()>;

#[derive(Clone)]
pub struct Replier {
    pub bot: AutoSend<Bot>,
    pub message_id: i32,
    pub chat_id: ChatId,
    pub lang: Lang,
}

impl Replier {
    pub fn from_message(bot: AutoSend<Bot>, message: &Message) -> Self {
        Self {
            bot,
            message_id: message.id,
            chat_id: message.chat.id,
            lang: Lang::from(message),
        }
    }

    pub fn from_callback_query(bot: AutoSend<Bot>, q: &CallbackQuery) -> Option<Self> {
        if let Some(message) = &q.message {
            let mut repl = Self::from_message(bot, &message);
            repl.lang = Lang::from(&q.from);
            Some(repl)
        } else {
            None
        }
    }

    pub fn user_id(&self) -> Option<i64> {
        if self.chat_id.is_user() {
            Some(self.chat_id.0)
        } else {
            None
        }
    }

    pub fn is_moderator(&self) -> bool {
        self.user_id()
            .map(|u| CONF.moderators.contains(&u))
            .unwrap_or_default()
    }

    pub fn send_text(&self, text: impl Translate) -> Reply {
        let text = text.translate(self.lang);
        self.bot
            .send_message(self.chat_id, text)
            .disable_web_page_preview(true)
    }

    pub fn send_html(&self, html: impl Translate) -> Reply {
        let html = html.translate(self.lang);
        self.send_text(html).parse_mode(ParseMode::Html)
    }

    pub fn edit_text(
        &self,
        text: impl Into<String>,
    ) -> <AutoSend<Bot> as Requester>::EditMessageText {
        self.bot
            .edit_message_text(self.chat_id, self.message_id, text)
    }
}
