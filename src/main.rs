use anyhow::Result;
use clap::Parser;

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
enum App {
    #[clap(name = "create-thumbs")]
    CreateThumbsSubcommand(CreateThumbsSubcommand),

    #[clap(name = "start-server")]
    StartServerSubcommand(StartServerSubcommand),
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
    }
}
