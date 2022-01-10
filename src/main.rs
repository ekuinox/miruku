use anyhow::Result;
use clap::Parser;

mod create_thumbs;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct CreateThumbsSubcommand {
    source: String,
    dest: String,
}

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct StartServerSubcommand {
    source: String,
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
            Ok(())
        },
        App::CreateThumbsSubcommand(s) => {
            dbg!(&s);

            use create_thumbs::*;
            use std::path::Path;
            let source = Path::new(&s.source);
            let dest = Path::new(&s.dest);
            let _ = create_thumbs(source, dest)?;
            Ok(())
        },
    }
}
