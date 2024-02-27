# Stage 1: Build environment
FROM ubuntu:22.04 AS builder

# Install Rust and other build dependencies
RUN apt-get update && \
    apt-get install -y curl build-essential && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Set the environment variable to ensure commands are executed in non-interactive mode
ENV PATH="/root/.cargo/bin:${PATH}"

# Copy the workspace source
WORKDIR /usr/src/frakt
COPY . .

# Build the application
RUN cargo build --release --bin cli

# Stage 2: The final image
FROM ubuntu:22.04
COPY --from=builder /usr/src/frakt/target/release/cli /usr/local/bin/cli
COPY entrypoint.sh /usr/local/entrypoint.sh
RUN chmod +x /usr/local/entrypoint.sh
COPY .env /usr/local/bin/.env

EXPOSE 8787
EXPOSE 8686

# Run the application
ENTRYPOINT [ "/usr/local/entrypoint.sh" ]
CMD ["cli"]
