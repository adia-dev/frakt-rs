use clap::Subcommand;

use self::{server::ServerCommand, worker::WorkerCommand};

pub mod server;
pub mod worker;

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 🚀 Start Server
    ///
    /// Initialize and run the server instance, managing workers and tasks.
    Server(ServerCommand),

    /// 👷 Worker Mode
    ///
    /// Launch a worker to perform assigned tasks and computations.
    Worker(WorkerCommand),
}
