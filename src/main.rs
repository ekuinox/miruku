#[macro_use]
extern crate anyhow;

use anyhow::Result;
use clap::Parser;

mod media;
mod thumbs;
mod server;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct CreateThumbsSubcommand {
    source: String,
    dest: String,
}

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct StartServerSubcommand {
    images_dir: String,
    thumbs_dir: String,
    port: u64,
}

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct GenerateMediaSubcommand {
    origin: String,
    dest: String,
}

#[derive(Parser, Debug)]
#[clap(about, version, author)]
enum App {
    #[clap(name = "create-thumbs")]
    CreateThumbsSubcommand(CreateThumbsSubcommand),

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
                images_dir: Path::new(&s.images_dir),
                thumbs_dir: Path::new(&s.thumbs_dir),
                port: s.port,
            };

            let _ = server.start().await?;

            Ok(())
        },
        App::CreateThumbsSubcommand(s) => {
            dbg!(&s);

            use thumbs::*;
            use std::path::Path;
            let source = Path::new(&s.source);
            let dest = Path::new(&s.dest);
            let _ = create_thumbs(source, dest)?;
            Ok(())
        },
        App::GenerateMediaSubcommand(s) => {
            use media::*;
            use std::path::Path;

            let origin = Path::new(&s.origin);
            let dest = Path::new(&s.dest);

            if origin.is_dir() {
                let medias = Media::generate_many(&origin, &dest).await?;

                dbg!(&medias);

                return Ok(())
            }

            let media = Media::generate(&origin, &dest).await?;

            dbg!(&media);

            Ok(())
        },
    }
}
