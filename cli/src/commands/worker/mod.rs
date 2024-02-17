use clap::Parser;

/// 👷 Worker Command
///
/// This command configures and launches a worker instance 🚧.
/// Workers perform specific tasks 🛠️ assigned by the server.
#[derive(Parser, Debug)]
#[command(name = "worker", about = "🚀 Start and configure a worker.", long_about = None)]
pub struct WorkerCommand {
    /// 📛 Worker name
    ///
    /// Unique identifier for the worker 🆔.
    /// If not set, a random name will be generated.
    #[arg(short, long, value_name = "NAME")]
    pub name: Option<String>,

    /// 📡 Server address
    ///
    /// The IP address of the server 🏢 to which the worker will connect.
    #[arg(short, long, value_name = "ADDRESS")]
    pub address: Option<String>,

    /// 🔌 Server port
    ///
    /// The port number of the server 🎚️ to which the worker will connect.
    #[arg(short, long, value_name = "PORT")]
    pub port: Option<u16>,

    /// 🔌 Replicas
    ///
    /// The numer of replicas to launch of a given worker
    #[arg(short, long, value_name = "COUNT")]
    pub count: Option<u32>,

    /// 🏋️‍♂️ Maximum workload
    ///
    /// Define the maximum workload that the worker can handle 📊.
    /// This could be in terms of tasks, computations, or data size.
    #[arg(long, value_name = "WORKLOAD")]
    pub maximal_work_load: Option<u32>,
}
