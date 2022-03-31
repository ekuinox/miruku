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
    StartServerSubcommand(StartServerSubcommand),

    #[clap(name = "generate-media")]
    GenerateMediaSubcommand(GenerateMediaSubcommand),
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = App::parse();
    log::debug!("{:#?}", args);

    env_logger::init();

    match args {
        App::StartServerSubcommand(s) => {
            use server::*;
            use std::path::Path;

            let server = Server {
                data_dir: Path::new(&s.data_dir),
                port: s.port,
            };

            let _ = server.start().await?;

            Ok(())
        }
        App::GenerateMediaSubcommand(s) => {
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
    }
}
