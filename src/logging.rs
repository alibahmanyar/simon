use log::{LevelFilter, debug};

pub fn setup() {
    // Initialize logger with timestamp and module path
    env_logger::Builder::new()
        .filter(None, LevelFilter::Info)
        .parse_env("SIMON_LOG")
        .target(env_logger::Target::Stdout)
        .init();

    // Print ASCII art banner
    let ver = env!("CARGO_PKG_VERSION");
    println!(
        "\n
███████ ██ ███    ███  ██████  ███    ██
██      ██ ████  ████ ██    ██ ████   ██
███████ ██ ██ ████ ██ ██    ██ ██ ██  ██
     ██ ██ ██  ██  ██ ██    ██ ██  ██ ██
███████ ██ ██      ██  ██████  ██   ████
                 v{ver}\n"
    );

    debug!("Logging initialized at {:?}", chrono::Local::now());
}
