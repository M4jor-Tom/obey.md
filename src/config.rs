pub const MAX_ITERATIONS: i32 = 10;
pub const HARD_MAX_ITERATIONS: i32 = 100;
pub const MAX_SCENE_DURATION: f64 = 60.0;
pub const MAX_KEYFRAMES_PER_CHANNEL: i32 = 10000;
pub const MAX_RETRIES: i32 = 3;
pub const CONTEXT_BUDGET_TOKENS: i32 = 128_000;

pub fn configure_logging(level: &str, log_file: Option<&str>) {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", level);
    }
    env_logger::init();

    if let Some(path) = log_file {
        match std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Warning: could not open log file {}: {}", path, e);
            }
        }
    }
}
