use std::env;
use std::time::Duration;

pub struct Args {
    pub workers: usize,
    pub max_connection_rate: usize,
    pub port: u16,
    pub address: String,
    pub client_request_timeout: std::time::Duration,
    pub download_sonnets: bool,
}

pub fn get_args() -> Args {
    let workers: usize = env::var("SERVER_WORKERS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1)
        });

    let max_connection_rate = env::var("SERVER_MAX_CONN_RATE")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(512);

    let port = env::var("SERVER_PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(8079);

    let address = env::var("SERVER_ADDRESS")
        .ok()
        .unwrap_or("0.0.0.0".to_string());

    let client_request_timeout: Duration = env::var("SERVER_CLIENT_REQUEST_TIMEOUT")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .map(Duration::from_secs)
        .unwrap_or(Duration::from_secs(600));

    let download_sonnets = env::var("DOWNLOAD_SONNETS")
        .ok()
        .and_then(|s| s.parse::<bool>().ok())
        .unwrap_or(false);

    Args {
        workers,
        max_connection_rate,
        port,
        address,
        client_request_timeout,
        download_sonnets,
    }
}
