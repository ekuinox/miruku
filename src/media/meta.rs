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
    pub media_id: MediaId,
    pub origin: String,
    pub visibility: MediaVisibility,
    pub date: Option<chrono::NaiveDateTime>,
    pub attributes: Option<Json<HashMap<String, String>>>,
}

#[derive(FromRow, Debug, Clone)]
struct OnlyMediaIdRow {
    pub media_id: MediaId,
}

impl MediaMeta {
    pub fn new(origin: String) -> Self {
        MediaMeta {
            origin,
            media_id: MediaId::new(),
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
        insert into metas (media_id, origin, visibility, date, attributes)
        values ($1, $2, $3, $4, $5)
        returning *
        "#)
            .bind(self.media_id.to_string())
            .bind(self.origin.to_string())
            .bind(self.visibility)
            .bind(self.date)
            .bind(self.attributes.as_ref())
            .fetch_one(conn)
            .await;
        println!("{:?}", _meta);

        Ok(())
    }

    pub async fn open(conn: &mut SqliteConnection, media_id: &String) -> Result<Self> {
        let meta = query_as("select * from metas where media_id = $1")
            .bind(media_id.to_string())
            .fetch_one(conn)
            .await?;
        Ok(meta)
    }

    pub async fn ids(conn: &mut SqliteConnection) -> Result<Vec<MediaId>> {
        let ids: Vec<OnlyMediaIdRow> = query_as("select media_id from metas")
            .fetch_all(conn)
            .await?;
        let ids: Vec<MediaId> = ids
            .into_iter()
            .map(|row| row.media_id.into())
            .collect();
        Ok(ids)
    }
}
