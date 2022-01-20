use anyhow::Result;
use serde::Serialize;
use sqlx::{prelude::*, types::Json, SqliteConnection, query_as};
use std::{collections::HashMap, ops::Deref, path::Path};

/// メディアのアクセスレベル
#[derive(Type, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
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
#[derive(Serialize, Type, Debug, Clone, PartialEq, Eq)]
#[sqlx(transparent)]
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
#[derive(FromRow, Debug, Clone)]
pub struct MediaMeta {
    pub id: MediaId,
    pub origin: String,
    pub visibility: MediaVisibility,
    pub date: Option<chrono::NaiveDateTime>,
    pub attributes: Option<Json<HashMap<String, String>>>,
}

impl MediaMeta {
    pub fn new(origin: String) -> Self {
        MediaMeta {
            origin,
            id: MediaId::new(),
            visibility: Default::default(),
            date: None,
            attributes: Default::default(),
        }
    }

    #[allow(dead_code)]
    pub fn make_public(self) -> Self {
        MediaMeta {
            visibility: MediaVisibility::Public,
            ..self
        }
    }

    #[allow(dead_code)]
    pub fn make_private(self) -> Self {
        MediaMeta {
            visibility: MediaVisibility::Private,
            ..self
        }
    }

    pub async fn save(&self, conn: &mut SqliteConnection) -> Result<()> {
        // とりあえず重複は考えない
        let _meta = query_as::<_, MediaMeta>(r#"
        insert into metas (id, origin, visibility, date, attributes)
        values ($1, $2, $3, $4)
        "#)
            .bind(self.id.to_string())
            .bind(self.origin.to_string())
            .bind(self.visibility)
            .bind(self.date)
            .bind(self.attributes.as_ref())
            .fetch_one(conn)
            .await?;

        Ok(())
    }

    pub async fn open(conn: &mut SqliteConnection, media_id: &String) -> Result<Self> {
        let meta = query_as("select * from metas where id = ?")
            .bind(media_id.to_string())
            .fetch_one(conn)
            .await?;
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
