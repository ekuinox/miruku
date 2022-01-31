use super::common::*;
use super::meta::*;
use anyhow::Result;
use chrono::Utc;
use sqlx::Connection;
use sqlx::SqliteConnection;
use std::path::Path;
use tokio::task;

// サムネイルの画像ファイル名
const THUMB_FILE_NAME: &'static str = "thumb.jpg";

// SQLite3データベースを返す
pub async fn create_connection(data_directory: &Path) -> Result<SqliteConnection> {
    let conn = SqliteConnection::connect(&format!(
        "sqlite://{}/db.sqlite3",
        data_directory.to_string_lossy()
    ))
    .await?;
    Ok(conn)
}

#[derive(Debug, Clone)]
pub struct Media {
    pub meta: MediaMeta,
}

impl From<MediaMeta> for Media {
    fn from(meta: MediaMeta) -> Self {
        Media { meta }
    }
}

pub struct MediaGenerateOption {}

impl Default for MediaGenerateOption {
    fn default() -> Self {
        MediaGenerateOption {}
    }
}

impl Media {
    /// ファイルを指定して生成する
    pub async fn generate(
        origin: &Path,
        data_directory: &Path,
        _option: &MediaGenerateOption,
    ) -> Result<Self> {
        use super::thumb::create_thumb;
        use tokio::fs::*;

        let mut conn = create_connection(data_directory).await?;

        // ファイルのハッシュ値を取得する
        let hashed = get_file_hash(&origin).await?;

        // ハッシュ値が一致している場合は生成しない
        if let Ok(meta) = MediaMeta::get_by_hashed(&mut conn, &hashed).await {
            // eprintln!("Already media created. file={:#?}", origin);
            return Ok(meta.into());
        }

        // 日付を取得する
        // exif -> file created at -> now とフォールバックしたい
        let date = if let Ok(date) = get_exif_date(&origin).await {
            date
        } else if let Ok(date) = get_file_created_date(&origin).await {
            date
        } else {
            Utc::now().naive_utc()
        };

        // generate meta data
        let meta = MediaMeta::new(origin.to_string_lossy().to_string(), hashed, date);
        let media_id = meta.media_id.clone();
        let _ = meta.save(&mut conn).await?;

        // media_id に応じたディレクトリのパス
        let media_directory = data_directory.join(MEDIA_DIRECTORY_NAME).join(&*media_id);

        // ディレクトリを掘っておく
        let _ = create_dir_all(&media_directory).await?;

        // generate thumbnail
        let dest = media_directory.join(THUMB_FILE_NAME);

        let source = origin.to_owned();
        let _ = task::spawn_blocking(move || create_thumb(&source, &dest)).await?;

        Ok(Media { meta })
    }

    /// ディレクトリを指定して読み込む
    pub async fn generate_many(
        source_directory: &Path,
        data_directory: &Path,
        option: &MediaGenerateOption,
    ) -> Result<Vec<Self>> {
        use indicatif::ProgressBar;
        use tokio_stream::{self as stream, StreamExt};

        // source のファイル一覧を取得
        let entries = get_image_filenames(source_directory);

        let entries = entries
            .iter()
            .map(|entry| Media::generate(&entry, data_directory, option));
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

        let path = data_directory
            .join(MEDIA_DIRECTORY_NAME)
            .join(&*self.meta.media_id)
            .join(THUMB_FILE_NAME);

        let mut file = File::open(&path).await?;
        let mut buf = Vec::<u8>::new();

        let _ = file.read_to_end(&mut buf).await?;

        Ok(buf)
    }

    /// オリジナルの画像を取得する
    pub async fn get_origin(&self) -> Result<Vec<u8>> {
        use tokio::fs::*;
        use tokio::io::AsyncReadExt;

        let mut file = File::open(Path::new(&self.meta.origin)).await?;
        let mut buf = Vec::<u8>::new();

        let _ = file.read_to_end(&mut buf).await?;

        Ok(buf)
    }
}

/// EXIF から 日付を取得する
async fn get_exif_date(path: &Path) -> Result<chrono::NaiveDateTime> {
    use chrono::NaiveDateTime;
    use exif::{In, Reader, Tag};
    use std::fs::File; // ここtoio化したい
    use std::io::BufReader;

    let file = File::open(path)?;
    let mut bufreader = BufReader::new(&file);
    let exifreader = Reader::new();
    let exif = exifreader.read_from_container(&mut bufreader)?;

    // カメラが違えば他のフィールドで取れる可能性もあるしうーん
    let field = match exif.get_field(Tag::DateTime, In::PRIMARY) {
        Some(field) => field.display_value().to_string(),
        None => bail!("not found"), // ここなんでunreachableなの？
    };

    // できればミリ秒までの精度が欲しいけどなあ
    let date = NaiveDateTime::parse_from_str(field.as_str(), "%Y-%m-%d %H:%M:%S")?;

    Ok(date)
}

/// ファイルのメタデータから日付を取得する
async fn get_file_created_date(path: &Path) -> Result<chrono::NaiveDateTime> {
    use chrono::NaiveDateTime;
    use std::time::UNIX_EPOCH;
    use tokio::fs::File;

    let file = File::open(path).await?;
    let meta = file.metadata().await?;

    let created = meta.created()?;
    let created = created.duration_since(UNIX_EPOCH)?;
    let created = match NaiveDateTime::from_timestamp_opt(
        created.as_secs() as i64,
        created.as_nanos() as u32,
    ) {
        Some(created) => created,
        _ => bail!("Err create NaiveDateTime"),
    };

    Ok(created)
}

/// パスで与えたファイルのハッシュ値を取得する
async fn get_file_hash(path: &Path) -> Result<Vec<u8>> {
    use sha2::{Digest, Sha512};
    use tokio::{fs::File, io::AsyncReadExt};

    let mut file = File::open(&path).await?;
    let mut buf = vec![];
    let _ = file.read_to_end(&mut buf).await?;

    let mut hasher = Sha512::new();
    hasher.update(&buf);

    let hashed = hasher.finalize();
    let hashed = hashed.into_iter().collect::<Vec<_>>();

    Ok(hashed)
}
