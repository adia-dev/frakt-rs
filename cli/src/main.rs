pub mod commands;

use clap::Parser;
use commands::{server::ServerCommand, worker::WorkerCommand, Cli, Commands};
use shared::{
    env, logger,
    networking::{server::ServerConfig, worker::Worker},
};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    env::init();
    logger::init_with_level(cli.log_level.to_string().as_str());

    match cli.command {
        Commands::Worker(args) => run_workers(args).await,
        Commands::Server(args) => run_server(args).await,
    }
}

async fn run_workers(args: WorkerCommand) {
    let address = args.address.unwrap_or_else(|| "localhost".to_string());
    let port = args.port.unwrap_or(8787);
    let maximal_work_load = args.maximal_work_load.unwrap_or(500);
    let count = args.count.unwrap_or(1);

    let worker_tasks: Vec<_> = (0..count)
        .map(|_| {
            let worker_address = address.clone();
            let worker_name = args
                .name
                .clone()
                .unwrap_or_else(|| format!("worker-{}", Uuid::new_v4()));
            tokio::spawn(async move {
                let worker = Worker::new(worker_name, maximal_work_load, worker_address, port);
                worker::run_worker(worker).await;
            })
        })
        .collect();

    // Await all worker tasks
    for task in worker_tasks {
        task.await.expect("Worker task failed");
    }
}

async fn run_server(args: ServerCommand) {
    let address = args.address.unwrap_or_else(|| "localhost".to_string());
    let port = args.port.unwrap_or(8787);
    let width = args.width.unwrap_or(300);
    let height = args.height.unwrap_or(300);
    let tiles = args.tiles.unwrap_or(4);
    let graphics = args.graphics.unwrap_or(false);
    let portal = args.portal.unwrap_or(false);

    let server_config = ServerConfig::new(address, port, width, height, tiles, graphics, portal);
    server::run_server(&server_config).await;
}
