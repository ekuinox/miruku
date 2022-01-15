use anyhow::Result;
use tokio::task;
use std::path::{Path, PathBuf};
use super::meta::*;
use super::common::*;

// サムネイルの画像ファイル名
const THUMB_FILE_NAME: &'static str = "thumb.jpg";

#[derive(Debug, Clone)]
pub struct Media {
    pub meta: MediaMeta,
}

impl From<MediaMeta> for Media {
    fn from(meta: MediaMeta) -> Self {
        Media { meta }
    }
}

pub struct MediaGenerateOption {
    /// ファイルをコピーした後、元のファイルを削除する
    pub is_remove_source: bool,
}

impl Default for MediaGenerateOption {
    fn default() -> Self {
        MediaGenerateOption {
            is_remove_source: true,
        }
    }
}

impl Media {
    /// ファイルを指定して生成する
    pub async fn generate(
        origin: &Path,
        data_directory: &Path,
        option: &MediaGenerateOption
    ) -> Result<Self> {
        use tokio::fs::*;
        use super::thumb::create_thumb;

        let media_directory = data_directory.join(MEDIA_DIRECTORY_NAME);

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
            .join(THUMB_FILE_NAME);

        let source = origin.to_owned();
        let _ = task::spawn_blocking(move || {
            create_thumb(&source, &dest)
        }).await?;

        // copy
        let dest = media_directory
            .join(&*media_id)
            .join(name);
        let _ = copy(origin, &dest).await?;

        if option.is_remove_source {
            remove_file(origin).await?;
        }

        Ok(media)
    }

    /// ディレクトリを指定して読み込む
    pub async fn generate_many(
        source_directory: &Path,
        data_directory: &Path,
        option: &MediaGenerateOption
    ) -> Result<Vec<Self>> {
        use tokio_stream::{self as stream, StreamExt};
        use indicatif::ProgressBar;

        // source のファイル一覧を取得
        let entries = get_image_filenames(source_directory)?;
        let entries = entries.into_iter()
            .map(|s| source_directory.join(s))
            .collect::<Vec<PathBuf>>();

        let entries = entries.iter().map(|entry|
            Media::generate(&entry, data_directory, option)
        );
        let mut medias = Vec::<Media>::with_capacity(entries.len());
        let mut stream = stream::iter(entries);

        let pb = ProgressBar::new(medias.capacity() as u64);

        while let Some(entry) = stream.next().await {
            if let Ok(entry) = entry.await {
                medias.push(entry);
                pb.inc(1);
            }
        }

        Ok(medias)
    }

    /// サムネイルを取得する
    pub async fn get_thumb(&self, data_directory: &Path) -> Result<Vec<u8>> {
        use tokio::fs::*;
        use tokio::io::AsyncReadExt;

        let path = data_directory.join(&*self.meta.id).join(THUMB_FILE_NAME);

        let mut file = File::open(&path).await?;
        let mut buf = Vec::<u8>::new();

        let _ = file.read_to_end(&mut buf).await?;

        Ok(buf)
    }

    /// オリジナルの画像を取得する
    pub async fn get_origin(&self, data_directory: &Path) -> Result<Vec<u8>> {
        use tokio::fs::*;
        use tokio::io::AsyncReadExt;

        let path = data_directory.join(&*self.meta.id).join(&self.meta.origin);

        let mut file = File::open(&path).await?;
        let mut buf = Vec::<u8>::new();

        let _ = file.read_to_end(&mut buf).await?;

        Ok(buf)
    }
}
