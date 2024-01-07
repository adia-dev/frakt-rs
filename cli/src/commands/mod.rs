use clap::Subcommand;

use self::{server::ServerCommand, worker::WorkerCommand};

pub mod server;
pub mod worker;

#[derive(Subcommand, Debug)]
pub enum Commands {
    Server(ServerCommand),
    Worker(WorkerCommand),
}
