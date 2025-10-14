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
        "Configuration: address={}, port={}, workers={}, max_connection_rate={}, timeout={:?}",
        args.address,
        args.port,
        args.workers,
        args.max_connection_rate,
        timeout
    );

    // Handle download flag
    if args.download_sonnets {
        log::info!("Download sonnets flag detected, downloading sonnets.txt...");
        if let Err(e) = mock_openai::common::download_sonnets().await {
            log::error!("Failed to download sonnets.txt: {}", e);
            eprintln!("Failed to download sonnets.txt: {}", e);
            std::process::exit(1);
        }
        log::info!("Sonnets download completed successfully");
    }

    log::info!("Initializing tokenizer and loading sonnets...");
    let _ = &mock_openai::common::TOKENIZED_OUTPUT;
    log::info!(
        "Tokenizer initialized successfully with {} tokens",
        *mock_openai::common::MAX_TOKENS
    );

    log::info!("Starting server on {}:{}", args.address, args.port);
    start_server(&args.address, args.port, args.token).await
}
