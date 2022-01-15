
use anyhow::Result;
use once_cell::sync::Lazy;
use regex::Regex;
use tokio::task;
use std::path::{Path, PathBuf};
use super::meta::*;

// JPEG画像をフィルタするための正規表現
pub static JPEG_FILE_EXT_REGEX: Lazy<Regex> = Lazy::new(
    || Regex::new(r"\.jpe?g$").expect("JPEG_FILE_EXT_REGEX is not valid")
);

#[derive(Debug, Clone)]
pub struct Media {
    pub meta: MediaMeta,
}

impl Media {
    /// ファイルを指定して生成する
    pub async fn generate(
        origin: &Path,
        data_directory: &Path,
    ) -> Result<Self> {
        use tokio::fs::*;
        use super::thumb::create_thumb;

        let media_directory = data_directory.join("media");

        let name = origin.file_name()
            .map(|s| s.to_str().map(|s| s.to_string()))
            .flatten();
        let name = match name {
            Some(name) => name,
            None => return Err(anyhow!("origin path is not satisfied")),
        };

        // generate meta data
        let meta = MediaMeta::new(name.clone());
        let media_id = meta.id.clone();
        let _ = meta.save(&media_directory).await?;

        let media = Media { meta };

        // generate thumbnail
        let dest = media_directory
            .join(&*media_id)
            .join("thumb.jpg");

        let source = origin.to_owned();
        let _ = task::spawn_blocking(move || {
            create_thumb(&source, &dest)
        }).await?;

        // copy
        let dest = media_directory
            .join(&*media_id)
            .join(name);
        let _ = copy(origin, &dest).await?;

        Ok(media)
    }

    /// ディレクトリを指定して読み込む
    pub async fn generate_many(
        source_directory: &Path,
        data_directory: &Path,
    ) -> Result<Vec<Self>> {
        // source のファイル一覧を取得
        let entries = get_image_filenames(source_directory)?;
        let entries = entries.into_iter()
            .map(|s| source_directory.join(s))
            .collect::<Vec<PathBuf>>();
        use tokio_stream::{self as stream, StreamExt};

        // TODO: 重複を避ける

        let entries = entries.iter().map(|entry| Media::generate(&entry, data_directory));
        let mut medias = Vec::<Media>::with_capacity(entries.len());
        let mut stream = stream::iter(entries);

        while let Some(entry) = stream.next().await {
            let ent = entry.await?;
            medias.push(ent)
        }

        Ok(medias)
    }
}


/// dir 下のファイル名をStringのリストで取得する
fn get_filenames(dir: &Path) -> Result<Vec<String>> {
    use std::fs::*;

    let entries = read_dir(dir)?;

    // ファイル一覧をStringで取得
    let entries = entries.into_iter()
        .flat_map(|entry| {
            if let Ok(entry) = entry {
                entry.file_name().to_str().map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect::<Vec<String>>();
    Ok(entries)
}

/// 画像(jpeg)ファイルのみをリストで取得する
fn get_image_filenames(dir: &Path) -> Result<Vec<String>> {
    let entries = get_filenames(dir)?;

    // jpegファイルでフィルタ
    let entries = entries.into_iter()
        .filter(|s| JPEG_FILE_EXT_REGEX.is_match(&s.to_lowercase()))
        .collect::<Vec<String>>();

    Ok(entries)
}
