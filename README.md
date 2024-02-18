# Frakt Playground Project

## Introduction

Frakt playground, this is a development project aimed at experimenting with network, rendering and playing around with fractals and zig.

## Features

- **Modular CLI Commands**: Configure and launch server and worker instances with custom parameters.
- **Distributed Computing**: Deploy multiple workers to perform assigned tasks in parallel.
- **Fractal Computation and Visualization**: Generate and render fractals using distributed workers.
- **Dynamic Task Allocation**: Workers request tasks dynamically, optimizing load distribution.
- **Real-time Visualization**: Graphical interface to visualize fractal computations in real-time.

## Project Structure

```bash
cli/ # CLI tool for managing server and workers
server/ # Server module managing tasks and workers
worker/ # Worker module for executing tasks
shared/ # Shared utilities and models for networking, logging, etc.
complex-rs/ # Complex number library used in fractal computations
```

Each module is designed to be standalone, with `shared` providing common functionalities used across the project.

## Getting Started

### Prerequisites

- Rust and Cargo
- Tokio for async runtime
- Maybe zig if you want to play with the zig code (TODO: make the damn zig repo public)

### Installation

Clone the repository and navigate into the project directory:

```bash
git clone <repository-url>
cd frakt
```

Build the project using Cargo:

```bash
cargo build --release
```

### Running the Server

Start the server with default settings:

```bash
cargo run -- server
```

Or customize its configuration:

```bash
cargo run -- server --address "localhost" --port 8080 --width 800 --height 600 --tiles 4
```

### Launching Workers

Start a worker instance:

```bash
cargo run -- worker
```

Customize worker configuration:

```bash
cargo run -- worker --name "worker1" --address "localhost" --port 8080 --count 2
```

## Usage

The CLI tool provides various commands and options to control the behavior of servers and workers. Use `--help` to explore all available commands and their descriptions.

### Exploring Fractals

After launching the server and workers, use the graphical interface to navigate and explore different fractal patterns. Adjust the view using keyboard shortcuts defined in the graphical module documentation.

## Contributing

Contributions are welcome! Please read the CONTRIBUTING.md for guidelines on how to contribute to the project.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
