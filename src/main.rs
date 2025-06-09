use mock_openai::start_server;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Initialize logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("warning"));
    let _ = &mock_openai::common::TOKENIZED_OUTPUT;

    let args = mock_openai::args::get_args();
    start_server(&args.address, args.port).await
}
