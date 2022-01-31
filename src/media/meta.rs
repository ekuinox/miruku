use anyhow::Result;
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::*, query_as, types::Json, SqliteConnection};
use std::{collections::HashMap, ops::Deref};

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

/// MediaId を検索するためのフィルタ
#[derive(Deserialize, Debug)]
pub struct IdsFilter {
    /// 検索開始する日付開始地点をミリ秒で指定する
    pub begin: Option<i64>,

    /// 検索開始する日付終了地点をミリ秒で指定する
    pub end: Option<i64>,

    /// 取得件数を指定する
    pub count: Option<u64>,
}

impl IdsFilter {
    /// フィルタをタプルに展開する
    /// `begin` が `None` の場合は現在時刻, `end` が `None` の場合は `0`, `count` が `None` の場合は `100` をデフォルトに使用する
    pub fn build(self) -> (NaiveDateTime, NaiveDateTime, u64) {
        fn to_naive_datetime(milli: i64) -> NaiveDateTime {
            NaiveDateTime::from_timestamp(milli / 1000, (milli % 1000 * 1000 * 1000) as u32)
        }
        let begin = self
            .begin
            .map(to_naive_datetime)
            .unwrap_or_else(|| Utc::now().naive_utc());

        let end = self
            .end
            .map(to_naive_datetime)
            .unwrap_or_else(|| NaiveDateTime::from_timestamp(0, 0));
        let count = self.count.unwrap_or(100);
        (begin, end, count)
    }
}

impl MediaId {
    /// `MediaId` を日付で降順に検索して取得する関数
    /// 成功すると `MediaId` のリストと最後の要素の日付を返す
    pub async fn filter(
        conn: &mut SqliteConnection,
        option: IdsFilter,
        include_private: bool,
    ) -> Result<(Vec<MediaId>, NaiveDateTime)> {
        use sqlx::*;

        let (begin, end, count) = option.build();

        let visibility = if include_private {
            MediaVisibility::Private
        } else {
            MediaVisibility::Public
        };

        let ids: Vec<MediaIdWithDateRow> = query_as(
            r#"
            select media_id, date, visibility from metas
            where date between ? and ? and visibility = ?
            order by date desc
            limit ?
            "#,
        )
        .bind(end.to_string())
        .bind(begin.to_string())
        .bind(visibility)
        .bind(count as i64)
        .fetch_all(conn)
        .await?;
        let last = ids.last().map(|row| row.date).unwrap_or(end);
        let ids: Vec<MediaId> = ids.into_iter().map(|row| row.media_id.into()).collect();

        Ok((ids, last))
    }
}

/// メタファイルの構造
#[derive(FromRow, Debug, Clone)]
pub struct MediaMeta {
    pub media_id: MediaId,
    pub origin: String,
    pub visibility: MediaVisibility,
    pub date: NaiveDateTime,
    pub hashed: Vec<u8>,
    pub attributes: Option<Json<HashMap<String, String>>>,
}

#[derive(FromRow, Debug, Clone)]
struct MediaIdWithDateRow {
    pub media_id: MediaId,
    pub date: NaiveDateTime,
}

impl MediaMeta {
    pub fn new(origin: String, hashed: Vec<u8>, date: NaiveDateTime) -> Self {
        MediaMeta {
            date,
            hashed,
            origin,
            media_id: MediaId::new(),
            visibility: Default::default(),
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
        let _ = query_as::<_, MediaMeta>(
            r#"
        insert into metas (media_id, origin, visibility, date, hashed, attributes)
        values ($1, $2, $3, $4, $5, $6)
        returning *
        "#,
        )
        .bind(self.media_id.to_string())
        .bind(self.origin.to_string())
        .bind(self.visibility)
        .bind(self.date)
        .bind(&self.hashed)
        .bind(self.attributes.as_ref())
        .fetch_one(conn)
        .await?;

        Ok(())
    }

    pub async fn open(conn: &mut SqliteConnection, media_id: &String) -> Result<Self> {
        let meta = query_as("select * from metas where media_id = $1")
            .bind(media_id.to_string())
            .fetch_one(conn)
            .await?;
        Ok(meta)
    }

    pub async fn get_by_hashed(conn: &mut SqliteConnection, hashed: &Vec<u8>) -> Result<Self> {
        let meta = query_as("select * from metas where hashed = $1")
            .bind(hashed)
            .fetch_one(conn)
            .await?;
        Ok(meta)
    }
}
