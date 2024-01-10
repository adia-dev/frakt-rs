pub mod commands;

use clap::Parser;
use commands::Commands;
use shared::networking::{worker::Worker, server::Server};
use uuid::Uuid;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Worker(args) => {
            let worker_name = match &args.name {
                Some(name) => name.to_owned(),
                None => format!("worker-{}", Uuid::new_v4()),
            };
            let maximal_work_load = match args.maximal_work_load {
                Some(maximal_work_load) => maximal_work_load,
                None => 500,
            };

            let address = match args.address {
                Some(address) => address,
                None => "localhost".to_string(),
            };

            let port = match args.port {
                Some(port) => port,
                None => 8787,
            };

            let worker = Worker::new(worker_name, maximal_work_load, address, port);
            // NOTE: the run_worker fn takes ownership of the worker
            worker::run_worker(worker).await;
        }
        Commands::Server(args) => {
            let address = match args.address {
                Some(address) => address,
                None => "localhost".to_string(),
            };

            let port = match args.port {
                Some(port) => port,
                None => 8787,
            };

            let width = match args.width {
                Some(width) => width,
                None => 300,
            };

            let height = match args.height {
                Some(height) => height,
                None => 300,
            };

            let server = Server::new(address, port, width, height);
            server::run_server(&server).await;
        }
    }
}
