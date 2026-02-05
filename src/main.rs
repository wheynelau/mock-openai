use clap::Parser;
use mock_openai::start_server;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = mock_openai::args::Args::parse();

    // Initialize logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("warning"));

    log::info!("Starting mock-openai server v{}", env!("CARGO_PKG_VERSION"));
    let timeout: std::time::Duration = args.client_request_timeout.into();

    log::info!(
        "Configuration: address={}, port={}, workers={}, max_connection_rate={}, timeout={:?}, inter_token_latency={}ms",
        args.address,
        args.port,
        args.workers,
        args.max_connection_rate,
        timeout,
        args.inter_token_latency
    );

    log::info!(
        "Tokens generated at build time: {}",
        mock_openai::generated::MAX_TOKENS
    );

    log::info!("Starting server on {}:{}", args.address, args.port);
    start_server(
        &args.address,
        args.port,
        args.token,
        args.inter_token_latency,
    )
    .await
}
