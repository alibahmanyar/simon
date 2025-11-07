mod alerts;
mod auth;
mod collect_info;
mod config;
mod db;
mod endpoints;
mod logging;
mod models;

use alerts::check_alerts;
use axum::{
    Router,
    routing::{delete, get, post},
};
use collect_info::detect_system_capabilities;
use db::db_update;
use endpoints::{
    add_alert, add_notif_method, browse_directory, delete_alert, delete_notif_method,
    download_file, fallback_handler, get_alert_vars, get_alerts, get_container_logs,
    get_file_content, get_notif_methods, get_serve_dirs, historical_data, req_info, serve_static,
    ws_handler_d, ws_handler_g, ws_handler_p,
};
use log::{debug, error, info};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use sysinfo::System;
use tokio::{self, time::Duration};
use tower_http::compression::CompressionLayer;

use crate::endpoints::capabilities_handler;

async fn sys_refresh(sys: Arc<Mutex<System>>, update_interval: u64) {
    loop {
        {
            let mut sys_write = sys.lock().unwrap();
            sys_write.refresh_cpu_usage();
            sys_write.refresh_memory();
        }
        tokio::time::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL).await;
        {
            let mut sys_write = sys.lock().unwrap();
            sys_write.refresh_cpu_usage();
            sys_write.refresh_memory();
        }
        tokio::time::sleep(Duration::from_secs(update_interval)).await;
    }
}

#[tokio::main]
async fn main() {
    logging::setup();

    // Parse command line arguments
    let mut config = config::parse_config();
    let update_interval = config.update_interval;
    info!("Update interval: {} seconds", update_interval);

    // Create system instance for the main thread and web API
    let sys = System::new();

    // Detect system capabilities
    config.system_capabilities = detect_system_capabilities(config.clone());

    let shared_sys = Arc::new(Mutex::new(sys));

    let bg_sys = shared_sys.clone();
    let db_sys = shared_sys.clone();

    // System refresh background task with restart on panic
    tokio::spawn(async move {
        loop {
            let result = tokio::task::spawn(sys_refresh(bg_sys.clone(), update_interval)).await;
            match result {
                Err(e) => {
                    error!("System refresh task panicked: {}", e);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    info!("Restarting system refresh task");
                    // Continue the loop to restart the task
                }
                _ => {
                    break; // This should not happen as sys_refresh runs indefinitely
                }
            }
        }
    });
    debug!("System refresh background task started");

    // Database update background task with restart on panic
    let db_path = config.db_path.clone();
    tokio::spawn(async move {
        loop {
            let db_path = db_path.clone();
            let db_sys = db_sys.clone();
            let result = tokio::task::spawn(async move { db_update(db_sys, &db_path).await }).await;
            match result {
                Err(e) => {
                    error!("Database update task panicked: {}", e);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    info!("Restarting database update task");
                    // Continue the loop to restart the task
                }
                _ => {
                    break; // This should not happen as db_update runs indefinitely
                }
            }
        }
    });
    debug!("Database update background task started");

    let db_path = config.db_path.clone();
    // Check alerts background task with restart on panic
    tokio::spawn(async move {
        loop {
            let db_path = db_path.clone();
            let result = tokio::task::spawn(async move { check_alerts(&db_path).await }).await;
            match result {
                Err(e) => {
                    error!("Check alerts task panicked: {}", e);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    info!("Restarting check alerts task");
                    // Continue the loop to restart the task
                }
                _ => {
                    break; // This should not happen as db_update runs indefinitely
                }
            }
        }
    });
    debug!("Alerts checking background task started");

    let mut app = Router::new()
        .route("/", get(serve_static))
        .route("/favicon.png", get(serve_static))
        .route("/Inter-Regular.woff", get(serve_static))
        .route("/Inter-Regular.woff2", get(serve_static))
        .route("/RobotoMono-Regular.woff", get(serve_static))
        .route("/RobotoMono-Regular.woff2", get(serve_static))
        .route("/api/capabilities", get(capabilities_handler))
        .route("/auth", get(serve_static))
        .route("/auth", post(auth::auth_handler))
        .route("/ws/g", get(ws_handler_g))
        .route("/ws/p", get(ws_handler_p))
        .route("/ws/d", get(ws_handler_d))
        .route("/container_logs/{continer_id}", get(get_container_logs))
        .route("/reqinfo", get(req_info))
        .route("/api/historical", get(historical_data))
        .route("/api/notif_methods", post(add_notif_method))
        .route("/api/notif_methods", get(get_notif_methods))
        .route("/api/notif_methods/{id}", delete(delete_notif_method))
        .route("/api/alerts", post(add_alert))
        .route("/api/alerts", get(get_alerts))
        .route("/api/alerts/{id}", delete(delete_alert))
        .route("/api/alert_vars", get(get_alert_vars))
        .route("/api/files/dirs", get(get_serve_dirs))
        .route("/api/files/browse", get(browse_directory))
        .route("/api/files/content", get(get_file_content))
        .route("/api/files/download", get(download_file))
        .fallback(fallback_handler)
        .with_state((shared_sys, config.clone()));

    if config.password_hash.is_some() {
        app = auth::apply_auth_middleware(app, config.clone());
        info!("Running with authentication");
    } else {
        info!("Running without authentication");
    }
    app = app.layer(CompressionLayer::new());

    info!(
        "Server running on http://{}:{}",
        config.address, config.port
    );

    let listener = tokio::net::TcpListener::bind(config.socket_address())
        .await
        .unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
