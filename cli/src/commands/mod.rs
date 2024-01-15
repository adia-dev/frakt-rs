use self::{server::ServerCommand, worker::WorkerCommand};
use clap::{Parser, Subcommand, ValueEnum};

pub mod server;
pub mod worker;

#[derive(Debug, ValueEnum, Clone)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl ToString for LogLevel {
    fn to_string(&self) -> String {
        format!("{:?}", self).to_lowercase()
    }
}

/// 🌟 Frakt CLI
///
/// The command center for managing and controlling the Frakt application 🎮.
/// Launch servers, workers, monitor performance, and tweak system configurations.
#[derive(Parser, Debug)]
#[command(author, version, about = "🔧 Frakt Command Line Interface", long_about = None)]
pub struct Cli {
    /// 📚 Subcommands
    ///
    /// Choose a specific operation mode for the Frakt application.
    #[clap(subcommand)]
    pub command: Commands,

    /// 📢 Log Level
    ///
    /// Set the verbosity level for logging output 📝.
    /// Options: error, warn, info, debug, trace.
    #[clap(long, default_value = "info", value_name = "LEVEL")]
    pub log_level: LogLevel,

    #[clap(short, long)]
    pub config: Option<std::path::PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 🚀 Start Server
    ///
    /// Initialize and run the server instance, managing workers and tasks.
    Server(ServerCommand),

    /// 👷 Worker Mode
    ///
    /// Launch one or multiple worker(s) to perform assigned tasks and computations.
    Worker(WorkerCommand),
}
