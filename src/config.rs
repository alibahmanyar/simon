use clap::Parser;
use rand::{self, Rng, distr::Alphanumeric};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr};

use crate::db;

#[derive(Parser, Debug, Clone, Serialize, Deserialize)]
#[command(version, about = "Simon - Simple Monitor")]
pub struct Config {
    /// Address to bind the server to
    #[arg(short, long, default_value = "0.0.0.0", env = "SIMON_ADDRESS")]
    pub address: IpAddr,

    /// Port to bind the server to
    #[arg(short, long, default_value = "30000", env = "SIMON_PORT")]
    pub port: u16,

    /// Update interval in seconds
    #[arg(short = 'T', long, default_value = "2", value_parser = clap::value_parser!(u64).range(1..=30), env = "SIMON_UPDATE_INTERVAL")]
    pub update_interval: u64,

    /// Authentication password bcrypt hash.
    /// If provided, authentication will be required. Leave empty to disable authentication.
    #[arg(short = 'H', long, env = "SIMON_PASSWORD_HASH")]
    pub password_hash: Option<String>,

    /// Database path
    #[arg(long, default_value = "./simon.db", env = "SIMON_DB_PATH")]
    pub db_path: String,

    /// JWT secret key for authentication tokens
    #[arg(skip)]
    pub jwt_secret: String,
}

impl Config {
    pub fn socket_address(&self) -> SocketAddr {
        SocketAddr::new(self.address, self.port)
    }
}

pub fn parse_config() -> Config {
    let mut config = Config::parse();

    if config.password_hash.is_some() {
        // check if valid bcrypt
        if !config.password_hash.as_ref().unwrap().starts_with("$2") {
            eprintln!("Invalid password: Password must be a valid bcrypt hash starting with '$2'");
            std::process::exit(1);
        }
    }

    let db = match db::Database::new(&config.db_path.clone()) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Failed to open database: {}", e);
            std::process::exit(1);
        }
    };

    match db.get_kv_str("jwt_secret") {
        Ok(Some(secret)) => {
            config.jwt_secret = secret;
        }
        Ok(None) => {
            if config.jwt_secret.is_empty() {
                // set to random value
                let mut rng = rand::rng();
                let jwt_secret: String = (&mut rng)
                    .sample_iter(Alphanumeric)
                    .take(63)
                    .map(char::from)
                    .collect();
                config.jwt_secret = jwt_secret.clone();

                if let Err(e) = db.set_kv_str("jwt_secret", &jwt_secret) {
                    eprintln!("Failed to write to database: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to read from database: {}", e);
            std::process::exit(1);
        }
    }
    
    config
}
