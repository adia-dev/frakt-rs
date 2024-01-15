pub mod commands;

use clap::{Parser, ValueEnum};
use commands::Commands;
use shared::{
    env, logger,
    networking::{server::Server, worker::Worker},
};
use uuid::Uuid;

#[derive(Debug, ValueEnum, Clone)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// 🌟 Frakt CLI
///
/// The command center for managing and controlling the Frakt application 🎮.
/// Launch servers, workers, monitor performance, and tweak system configurations.
#[derive(Parser, Debug)]
#[command(author, version, about = "🔧 Frakt Command Line Interface", long_about = None)]
struct Cli {
    /// 📚 Subcommands
    ///
    /// Choose a specific operation mode for the Frakt application.
    #[clap(subcommand)]
    command: Commands,

    /// 📢 Log Level
    ///
    /// Set the verbosity level for logging output 📝.
    /// Options: error, warn, info, debug, trace.
    #[clap(long, default_value = "info", value_name = "LEVEL")]
    log_level: String,

    #[clap(short, long)]
    config: Option<std::path::PathBuf>,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    env::init();
    logger::init_with_level(cli.log_level.as_str());

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

            let count = match args.count {
                Some(count) => count,
                None => 1,
            };

            let mut handles = Vec::new();
            for _ in 0..=count {
                let address = address.clone();
                let worker_name = format!("worker-{}", Uuid::new_v4());
                let handle = tokio::spawn(async move {
                    let worker = Worker::new(worker_name, maximal_work_load, address, port);
                    worker::run_worker(worker).await;
                });
                handles.push(handle);
            }

            // Wait for all the tasks to complete
            for handle in handles {
                handle.await.expect("Task failed");
            }
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
