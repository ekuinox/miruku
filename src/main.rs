use anyhow::Result;
use clap::Parser;

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
        },
        App::CreateThumbsSubcommand(s) => {
            dbg!(&s);
        },
    };

    Ok(())
}
