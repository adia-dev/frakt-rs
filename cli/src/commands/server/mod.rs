use clap::Parser;

#[derive(Parser, Debug)]
pub struct ServerCommand {
    #[arg(short, long)]
    pub address: Option<String>,

    #[arg(short, long)]
    pub port: Option<u16>,
}
