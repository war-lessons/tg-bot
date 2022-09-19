use sqlx::{migrate, PgPool};
use std::sync::{Arc, Mutex};
use teloxide::prelude::*;
use war_lessons_bot::{
    add_lesson, eprint_error, init_logging, log_error, message_text, start_keyboard, Error,
    LessonReadOptions, Replier, ReplyResult, Result, SetLessonStatus, SpamTokenGenerator, CONF,
    TEXT,
};

#[tokio::main]
async fn main() -> Result<()> {
    init_logging().map_err(|e| {
        eprint_error(&e);
        e
    })?;
    run().await.map_err(|e| {
        log_error(&e);
        e
    })
}

async fn run() -> Result<()> {
    let pool = PgPool::connect(&CONF.database_url)
        .await
        .map_err(Error::CreatePgPool)?;
    migrate!().run(&pool).await.map_err(Error::Migrate)?;

    let bot = Bot::new(&CONF.teloxide_token).auto_send();
    let spam_gen = Arc::new(Mutex::new(SpamTokenGenerator::new(
        CONF.spam_token_lifetime,
    )));

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handler))
        .branch(Update::filter_callback_query().endpoint(callback_handler));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![pool, spam_gen])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
    Ok(())
}

async fn message_handler(
    message: Message,
    bot: AutoSend<Bot>,
    pool: PgPool,
    spam_gen: Arc<Mutex<SpamTokenGenerator>>,
) -> ReplyResult {
    let repl = Replier::from_message(bot, &message);
    if let Some(text) = message_text(&message) {
        if text.starts_with('/') {
            handle_command(&pool, &repl, text).await?;
        } else if let Some(user_id) = repl.user_id() {
            let spam_token = spam_gen.lock().expect("spam_gen.lock").generate(user_id);
            add_lesson(&pool, &repl, &spam_token, text).await?;
        } else {
            repl.send_text("The bot works in private chats only")
                .await?;
        }
    } else {
        repl.send_text(&TEXT.text_only).await?;
    }
    Ok(())
}

async fn callback_handler(q: CallbackQuery, bot: AutoSend<Bot>, pool: PgPool) -> ReplyResult {
    if let (Some(cmd), Some(message)) = (q.data, q.message) {
        let repl = Replier::from_message(bot, &message);

        if let Some(opts) = SetLessonStatus::from_command(&cmd) {
            if repl.is_moderator() {
                opts.reply(&pool, &repl).await?;
                repl.bot
                    .answer_callback_query(q.id)
                    .text("Lesson status updated")
                    .await?;
            } else {
                repl.bot
                    .answer_callback_query(q.id)
                    .text("Forbidden")
                    .await?;
            }
        } else if cmd.starts_with('/') {
            handle_command(&pool, &repl, &cmd).await?;
            repl.bot.answer_callback_query(q.id).await?;
        } else {
            repl.bot
                .answer_callback_query(q.id)
                .text("Unknown callback query")
                .await?;
        }
    }
    Ok(())
}

async fn handle_command(pool: &PgPool, repl: &Replier, text: &str) -> ReplyResult {
    if text == "/start" || text == "/help" {
        repl.send_html(&TEXT.help_message)
            .reply_markup(start_keyboard(pool, repl.lang, repl.is_moderator()).await)
            .await?;
    } else if text == "/add" {
        repl.send_text(&TEXT.add_lesson_message).await?;
    } else if let Some(opts) = LessonReadOptions::from_command(text) {
        opts.reply(pool, repl).await?;
    } else {
        repl.send_text(&TEXT.unknown_command).await?;
    };
    Ok(())
}
