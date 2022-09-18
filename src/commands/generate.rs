use std::error::Error;
use std::io::Cursor;
use std::sync::Arc;

use async_trait::async_trait;
use image::{ImageFormat, ImageOutputFormat};
use reqwest::StatusCode;
use tgbotapi::requests::ParseMode;
use tgbotapi::FileType;

use super::Command;
use crate::api_methods::SendPhoto;
use crate::apis::craiyon;
use crate::utils::{donate_markup, escape_markdown, format_duration, image_collage, Context};

// yes, people generated all of these
const DISALLOWED_WORDS: [&str; 37] = [
    "abuse", "anus", "ass", "bikini", "boob", "booba", "boobs", "braless", "breast", "breasts",
    "butt", "butts", "cum", "dick", "doujin", "erotic", "hentai", "incest", "lingerie", "loli",
    "lolicon", "lolis", "naked", "nhentai", "nude", "penis", "porn", "porno", "rape", "sex",
    "sexy", "shota", "shotacon", "slut", "tits", "underage", "xxx",
];

pub struct Generate;

#[async_trait]
impl Command for Generate {
    fn name(&self) -> &str {
        "generate"
    }

    async fn execute(
        &self,
        ctx: Arc<Context>,
        arguments: Option<String>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let prompt = if let Some(arguments) = arguments {
            arguments
        } else {
            ctx.missing_argument("prompt to generate").await;
            return Ok(());
        };

        if let Some(issue) = check_prompt(&prompt) {
            log::warn!("Prompt rejected: {issue:?}");
            ctx.reply(issue).await?;
            return Ok(());
        }

        let status_msg = ctx.reply(format!("Generating {prompt}…")).await?;

        match craiyon::generate(ctx.http_client.clone(), &prompt).await {
            Ok(result) => {
                let images = result
                    .images
                    .into_iter()
                    .flat_map(|image| {
                        image::load_from_memory_with_format(&image, ImageFormat::Jpeg)
                    })
                    .collect::<Vec<_>>();
                let image = image_collage(images, 3, 8);
                let mut buffer = Cursor::new(Vec::new());
                image.write_to(&mut buffer, ImageOutputFormat::Png).unwrap();

                ctx.api
                    .make_request(&SendPhoto {
                        chat_id: ctx.message.chat_id(),
                        photo: FileType::Bytes("image.png".to_string(), buffer.into_inner()),
                        caption: Some(format!(
                            "Generated from prompt: *{}* in {}\\.",
                            escape_markdown(prompt),
                            format_duration(result.duration)
                        )),
                        parse_mode: Some(ParseMode::MarkdownV2),
                        reply_to_message_id: Some(ctx.message.message_id),
                        allow_sending_without_reply: Some(true),
                        reply_markup: Some(donate_markup(
                            "🖍️ Craiyon",
                            "https://www.craiyon.com/donate",
                        )),
                    })
                    .await?;
            }
            Err(err) => {
                ctx.reply(format!(
                    "zjebalo sie: {}",
                    err.status().unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
                ))
                .await?;
            }
        };

        ctx.delete_message(&status_msg).await?;

        Ok(())
    }
}

pub(super) fn check_prompt<S: AsRef<str>>(prompt: S) -> Option<&'static str> {
    let prompt = prompt.as_ref();
    if prompt.chars().count() > 1024 {
        Some("This prompt is too long.")
    } else if prompt.lines().count() > 5 {
        Some("This prompt has too many lines.")
    } else if is_prompt_suspicious(prompt) {
        Some("This prompt is sus.")
    } else {
        None
    }
}

fn is_prompt_suspicious<S: AsRef<str>>(text: S) -> bool {
    text.as_ref()
        .to_lowercase()
        .split(|c: char| !c.is_alphabetic())
        .any(|w| DISALLOWED_WORDS.contains(&w))
}
