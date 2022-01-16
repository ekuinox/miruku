#[macro_use]
extern crate anyhow;

use anyhow::Result;
use clap::Parser;

mod media;
mod server;

const DEFAULT_DATA_DIR: &'static str = "./data";
const DEFAULT_SERVER_PORT: &'static str = "9999";

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

    #[clap(long)]
    without_remove: bool,
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
    dbg!(&args);

    match args {
        App::StartServerSubcommand(s) => {
            dbg!(&s);

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

            let origin = Path::new(&s.origin);
            let dest = Path::new(&s.dest);

            let option = MediaGenerateOption {
                is_remove_source: !s.without_remove,
            };

            if origin.is_dir() {
                let medias = Media::generate_many(&origin, &dest, &option).await?;

                dbg!(&medias);

                return Ok(());
            }

            let media = Media::generate(&origin, &dest, &option).await?;

            dbg!(&media);

            Ok(())
        }
    }
}
