use mock_openai::start_server;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = mock_openai::args::get_args();

    // Handle download flag
    if args.download_sonnets {
        if let Err(e) = mock_openai::common::download_sonnets().await {
            eprintln!("Failed to download sonnets.txt: {}", e);
            std::process::exit(1);
        }
        return Ok(());
    }

    // Initialize logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("warning"));
    let _ = &mock_openai::common::TOKENIZED_OUTPUT;

    start_server(&args.address, args.port).await
}
