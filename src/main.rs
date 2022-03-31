#[macro_use]
extern crate anyhow;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

mod media;
mod server;

const DEFAULT_DATA_DIR: &str = "./data";
const DEFAULT_SERVER_PORT: &str = "9999";

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct StartServerSubcommand {
    #[clap(default_value = DEFAULT_DATA_DIR)]
    data_dir: String,

    #[clap(default_value = DEFAULT_SERVER_PORT)]
    port: u64,
}

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct GenerateMediaSubcommand {
    origin: String,

    #[clap(default_value = DEFAULT_DATA_DIR)]
    dest: String,

    #[clap(short = 'w')]
    watch: bool,
}

#[derive(Parser, Debug)]
#[clap(about, version, author)]
enum App {
    #[clap(name = "start-server")]
    StartServer(StartServerSubcommand),

    #[clap(name = "generate-media")]
    GenerateMedia(GenerateMediaSubcommand),

    /// データベースに記録した時刻を Local に直す
    #[clap(name = "fix-date")]
    FixDate { database_path: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = App::parse();
    log::debug!("{:#?}", args);

    env_logger::init();

    match args {
        App::StartServer(s) => {
            use server::*;
            use std::path::Path;

            let server = Server {
                data_dir: Path::new(&s.data_dir),
                port: s.port,
            };

            let _ = server.start().await?;

            Ok(())
        }
        App::GenerateMedia(s) => {
            use media::*;
            use std::path::Path;

            // イベントのパスからMediaを生成する
            async fn from_event_path(origin: PathBuf, dest: &Path, option: &MediaGenerateOption) {
                if !media::common::is_target(&origin) {
                    return;
                }
                if let Ok(origin) = origin.canonicalize() {
                    log::info!("start origin ({:#?})", origin);
                    match Media::generate(&origin, dest, option).await {
                        Ok(media) => log::info!("{:#?}: OK", media.meta.origin),
                        Err(e) => log::info!("{:#?}: {:?}", origin, e),
                    }
                }
            }

            let origin = Path::new(&s.origin);
            let dest = Path::new(&s.dest);

            let option = MediaGenerateOption {};

            if s.watch {
                use notify::{
                    watcher,
                    DebouncedEvent::{Create, Write},
                    RecursiveMode, Watcher,
                };
                use std::sync::mpsc::channel;
                use std::time::Duration;

                let (tx, rx) = channel();

                let mut watcher = watcher(tx, Duration::from_secs(5))?;
                watcher.watch(&origin, RecursiveMode::Recursive)?;

                loop {
                    match rx.recv() {
                        Ok(Write(origin)) => from_event_path(origin, dest, &option).await,
                        Ok(Create(origin)) => from_event_path(origin, dest, &option).await,
                        Ok(_event) => {}
                        Err(e) => log::debug!("watch error: {:?}", e),
                    }
                }
            }

            if origin.is_dir() {
                let medias = Media::generate_many(origin, dest, &option).await?;

                log::debug!("{:#?}", medias);

                return Ok(());
            }

            let media = Media::generate(origin, dest, &option).await?;

            log::debug!("{:#?}", media);

            Ok(())
        }
        App::FixDate { database_path } => fix_date(&database_path).await,
    }
}

async fn fix_date(data_dir: &str) -> Result<()> {
    use chrono::prelude::*;
    use media::*;
    use sqlx::{prelude::*, query_as, SqliteConnection};

    #[derive(FromRow, Debug, Clone)]
    struct MediaIdWithDateRow {
        pub media_id: MediaId,
        pub date: NaiveDateTime,
    }

    let mut conn = SqliteConnection::connect(data_dir).await?;

    let medias: Vec<MediaIdWithDateRow> = query_as(
        r#"
        select media_id, date, visibility from metas
        order by date desc
        "#,
    )
    .fetch_all(&mut conn)
    .await?;

    for MediaIdWithDateRow { media_id, date } in medias {
        let local_date = Local.from_utc_datetime(&date).naive_utc();
        const QUERY: &str = r#"
            update metas
            set date = ?
            where media_id = ?
        "#;
        let _ = sqlx::query(QUERY)
            .bind(local_date)
            .bind(media_id)
            .execute(&mut conn)
            .await?;
    }

    Ok(())
}
