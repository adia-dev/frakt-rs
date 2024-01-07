use clap::Parser;

#[derive(Parser, Debug)]
pub struct WorkerCommand {
    #[arg(short, long)]
    pub name: Option<String>,

    #[arg(short, long)]
    pub address: Option<String>,

    #[arg(short, long)]
    pub port: Option<u16>,

    #[arg(long)]
    pub maximal_work_load: Option<u32>,
}
