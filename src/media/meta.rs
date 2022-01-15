use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, ops::Deref, path::Path};

/// メタファイルの名前
const META_DATA_FILE_NAME: &'static str = "meta.toml";

/// メディアのアクセスレベル
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaVisibility {
    Private,
    Public,
}

impl Default for MediaVisibility {
    fn default() -> Self {
        // デフォルトはプライベートにする
        MediaVisibility::Private
    }
}

/// メディアのID
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct MediaId(String);

impl MediaId {
    pub fn new() -> Self {
        use uuid::Uuid;
        MediaId(Uuid::new_v4().to_string())
    }
}

impl From<String> for MediaId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

impl Deref for MediaId {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// メタファイルの構造
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MediaMeta {
    pub id: MediaId,
    pub origin: String,
    pub visibility: MediaVisibility,
    pub attributes: HashMap<String, String>,
}

impl MediaMeta {
    pub fn new(origin: String) -> Self {
        MediaMeta {
            origin,
            id: MediaId::new(),
            visibility: Default::default(),
            attributes: Default::default(),
        }
    }

    pub fn make_public(self) -> Self {
        MediaMeta {
            visibility: MediaVisibility::Public,
            ..self
        }
    }

    pub fn make_private(self) -> Self {
        MediaMeta {
            visibility: MediaVisibility::Private,
            ..self
        }
    }

    pub async fn save(&self, data_directory: &Path) -> Result<()> {
        use tokio::fs::*;
        use tokio::io::AsyncWriteExt;

        let media_directory = data_directory.join(&*self.id);

        let _ = create_dir_all(&media_directory).await?;

        let meta_path = media_directory.join(META_DATA_FILE_NAME);

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&meta_path)
            .await?;

        let serialized = toml::to_vec(&self)?;

        let _ = file.write_all(&serialized).await?;

        Ok(())
    }

    pub async fn open(data_directory: &Path, media_id: &String) -> Result<Self> {
        use tokio::fs::File;
        use tokio::io::AsyncReadExt;

        let meta_path = data_directory.join(&*media_id).join(META_DATA_FILE_NAME);

        let mut file = File::open(meta_path).await?;
        let mut buf = Vec::<u8>::new();
        let _ = file.read_to_end(&mut buf).await?;

        let meta = toml::from_slice::<Self>(&buf)?;

        Ok(meta)
    }

    pub async fn ids(data_directory: &Path) -> Result<Vec<MediaId>> {
        use tokio::fs::*;

        let mut entries = read_dir(data_directory).await?;
        let mut ids = Vec::<MediaId>::new();

        while let Ok(Some(entry)) = entries.next_entry().await {
            // ディレクトリであれば、メディアの各ファイルが入っているものとみなす
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str().map(|s| s.to_string()) {
                    ids.push(name.into());
                }
            }
        }

        Ok(ids)
    }
}
