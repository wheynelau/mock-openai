use clap::Parser;
use duration_string::DurationString;

#[derive(Parser, Debug)]
#[command(name = "mock-openai")]
#[command(about = "A mock OpenAI API server for testing purposes")]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Args {
    /// Number of worker threads to spawn
    #[arg(short, long, default_value_t = {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    })]
    pub workers: usize,

    /// Maximum number of connections per second
    #[arg(long, default_value = "512")]
    pub max_connection_rate: usize,

    /// Port to listen on
    #[arg(short, long, default_value = "8079")]
    pub port: u16,

    /// Address to bind to
    #[arg(short, long, default_value = "0.0.0.0")]
    pub address: String,

    /// Client request timeout (e.g., "600s", "10m", "1h", "30min")
    #[arg(
        long,
        default_value = "600s",
        value_parser = clap::value_parser!(DurationString)
    )]
    pub client_request_timeout: DurationString,

    /// Download sonnets.txt and exit
    #[arg(long)]
    pub download_sonnets: bool,
}
