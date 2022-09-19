use crate::{internal_error, Replier, ReplyResult, CONF, TEXT};
use sqlx::{query, PgPool};
use time::OffsetDateTime;

pub async fn add_lesson(
    pool: &PgPool,
    repl: &Replier,
    spam_token: &str,
    text: &str,
) -> ReplyResult {
    match is_flood(pool, spam_token).await {
        Err(e) => {
            repl.send_text(internal_error(&e)).await?;
        }
        Ok(Some(wait_for)) => {
            repl.send_text(
                TEXT.flood
                    .to(repl.lang)
                    .replace("{}", &wait_for.to_string()),
            )
            .await?;
        }
        Ok(None) => {
            if let Err(e) = save_message(pool, spam_token, text).await {
                repl.send_text(internal_error(&e)).await?;
            } else {
                repl.send_text(&TEXT.lesson_saved).await?;
            };
        }
    };
    Ok(())
}

/// Returns a number of seconds to wait before a next message
async fn is_flood(pool: &PgPool, spam_token: &str) -> sqlx::Result<Option<usize>> {
    if CONF.rate_limit_messages == 0 {
        return Ok(None);
    }

    let now = OffsetDateTime::now_utc();
    let from = now - CONF.rate_limit_duration;
    let times = query!(
        r#"
        SELECT created_at
        FROM lesson
        WHERE spam_token=$1
          AND created_at > $2
        ORDER BY created_at DESC
        "#,
        spam_token,
        from,
    )
    .map(|row| row.created_at)
    .fetch_all(pool)
    .await?;

    if times.len() < CONF.rate_limit_messages {
        return Ok(None);
    }

    let left = *times.last().unwrap() - from;
    Ok(Some(left.as_seconds_f32().ceil().abs() as usize))
}

async fn save_message(pool: &PgPool, spam_token: &str, text: &str) -> sqlx::Result<()> {
    query!(
        "INSERT INTO lesson (text, spam_token) VALUES ($1, $2)",
        text,
        spam_token
    )
    .execute(pool)
    .await?;
    Ok(())
}
