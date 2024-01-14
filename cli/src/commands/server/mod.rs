use clap::Parser;

/// 🖥️ Server Command
///
/// This command is used to configure and 🚀 start the server.
#[derive(Parser, Debug)]
#[command(name = "server", about = "🚀 Start and configure the server.", long_about = None)]
pub struct ServerCommand {
    /// 📌 Server IP address
    ///
    /// Specify the IP address 🌐 where the server will listen for incoming connections.
    /// If not set, the server will listen on all available interfaces.
    #[arg(short, long, value_name = "ADDRESS")]
    pub address: Option<String>,

    /// 🚪 Server port
    ///
    /// Define the port number 🎛️ on which the server will listen.
    /// Default is 8080 if not specified.
    #[arg(short, long, value_name = "PORT")]
    pub port: Option<u16>,

    /// 📏 Server width
    ///
    /// Set the width for the server's operational parameters 📐.
    /// This might represent the width of a window or a grid, depending on context.
    #[arg(long, value_name = "WIDTH")]
    pub width: Option<u16>,

    /// 📐 Server height
    ///
    /// Set the height for the server's operational parameters 🧱.
    /// Similar to width, this parameter depends on the specific use case.
    #[arg(long, value_name = "HEIGHT")]
    pub height: Option<u16>,
}
