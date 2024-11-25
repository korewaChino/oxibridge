use std::path::Path;

use crate::core;
use teloxide::{
    net::Download,
    prelude::*,
    types::{self, MediaKind, MessageKind, PhotoSize},
};
use tokio::fs;
use tracing::*;

#[instrument(skip(bot, m))]
pub async fn to_core_message(bot: Bot, m: Message) -> color_eyre::Result<core::Message> {
    let tg_author = m.from.as_ref().unwrap();
    let core_author = to_core_author(bot.clone(), &tg_author).await?;

    let (content, attachments) = match &m.kind {
        MessageKind::Common(common) => match &common.media_kind {
            MediaKind::Text(text) => (text.text.to_owned(), vec![]),
            MediaKind::Photo(photo) => {
                let attachment = photo_to_core_file(bot.clone(), &photo.photo).await?;
                (
                    photo.caption.clone().unwrap_or("".to_owned()),
                    vec![core::Attachment {
                        file: attachment,
                        spoilered: photo.has_media_spoiler,
                    }],
                )
            }
            _ => ("Unknown media attached".to_owned(), vec![]),
        },
        _ => ("Unknown message kind".to_owned(), vec![]),
    };

    Ok(core::Message {
        author: core_author,
        content: content,
        attachments: attachments,
    })
}

#[instrument(skip(bot))]
async fn to_core_author(bot: Bot, author: &types::User) -> color_eyre::Result<core::Author> {
    let photos = bot.get_user_profile_photos(author.id).await?;
    let photo = photos.photos.get(0);

    let core_file = match photo {
        Some(photo) => match photo_to_core_file(bot, &photo).await {
            Ok(file) => Some(file),
            Err(_) => None,
        },
        None => None,
    };

    Ok(core::Author {
        display_name: author.full_name(),
        username: author.username.clone().unwrap_or("Unknown".to_string()),
        avatar: core_file,
    })
}

pub async fn photo_to_core_file(
    bot: Bot,
    photo: &Vec<PhotoSize>,
) -> color_eyre::Result<core::File> {
    let photo = photo.get(photo.len() - 1).unwrap();
    let file = bot.get_file(&photo.file.id).await?;
    to_core_file(bot, &file).await
}

#[instrument(skip(bot))]
pub async fn to_core_file(
    bot: Bot,
    file: &teloxide::types::File,
) -> color_eyre::Result<core::File> {
    let path = core::get_tmp_dir()?.join(format!(
        "{}.{}",
        &file.unique_id,
        Path::new(&file.path).extension().unwrap().to_str().unwrap()
    ));
    let mut dst = fs::File::create(&path).await?;
    bot.download_file(&file.path, &mut dst).await?;

    Ok(core::File {
        name: file.path.replace("/", "_"),
        path: path,
    })
}