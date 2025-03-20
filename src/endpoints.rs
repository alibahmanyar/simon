use crate::collect_info;
use crate::config::Config;
use crate::db::Database;
use crate::models::{self, NotificationMethod};
use axum::Json;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::{
    extract::{ConnectInfo, State, WebSocketUpgrade},
    http::HeaderMap,
    response::{Html, IntoResponse},
};
use bollard::container::LogsOptions;
use futures::StreamExt;
use log::{debug, error, info, warn};
use models::HistoricalQueryOptions;
use serde_json;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use sysinfo::System;

use axum::extract::rejection::JsonRejection;
use tokio::{
    self,
    time::{Duration, interval},
};

// Embed the index.html file directly into the binary at compile time
const INDEX_HTML: &str = include_str!("../web/build/index.html");
const AUTH_HTML: &str = include_str!("../web/build/auth.html");
const FAVICON: &[u8] = include_bytes!("../web/build/favicon.png");
const FONT_1: &[u8] = include_bytes!("../web/build/Inter-Regular.woff");
const FONT_2: &[u8] = include_bytes!("../web/build/Inter-Regular.woff2");
const FONT_3: &[u8] = include_bytes!("../web/build/RobotoMono-Regular.woff");
const FONT_4: &[u8] = include_bytes!("../web/build/RobotoMono-Regular.woff2");

// Handlers to serve the embedded static stuff
pub async fn serve_index() -> impl IntoResponse {
    Html(INDEX_HTML)
}
pub async fn serve_auth() -> impl IntoResponse {
    Html(AUTH_HTML)
}
pub async fn serve_favicon() -> impl IntoResponse {
    (StatusCode::OK, FAVICON)
}
pub async fn serve_font_1() -> impl IntoResponse {
    (StatusCode::OK, FONT_1)
}
pub async fn serve_font_2() -> impl IntoResponse {
    (StatusCode::OK, FONT_2)
}
pub async fn serve_font_3() -> impl IntoResponse {
    (StatusCode::OK, FONT_3)
}
pub async fn serve_font_4() -> impl IntoResponse {
    (StatusCode::OK, FONT_4)
}

pub async fn req_info(ConnectInfo(addr): ConnectInfo<SocketAddr>, headers: HeaderMap) -> String {
    let headers_str = headers
        .iter()
        .map(|(k, v)| format!("{}: {}", k, v.to_str().unwrap()))
        .collect::<Vec<String>>()
        .join("\n");

    info!("Request info from IP: {}", addr);
    debug!("Headers: {}", headers_str);

    format!("IP: {}\n\nHeaders:\n{}", addr, headers_str)
}

// docker
pub async fn ws_handler_d(
    ws: WebSocketUpgrade,
    State((_, config)): State<(Arc<Mutex<System>>, Config)>,
) -> impl IntoResponse {
    debug!("Docker websocket connection requested");
    ws.on_upgrade(move |socket| handle_socket_d(socket, config.update_interval))
}

async fn handle_socket_d(mut socket: WebSocket, ws_interval: u64) {
    debug!("Docker websocket connection established");
    let mut interval = interval(Duration::from_secs(ws_interval));

    let mut docker_accessible = true;
    loop {
        let json_string = match collect_info::get_docker_containers().await {
            Some(info) => serde_json::to_string(&info).unwrap(),
            None => {
                warn!("Can't get docker containers info");
                docker_accessible = false;
                String::from("null")
            }
        };
        if socket
            .send(Message::Binary({
                let mut encoder =
                    flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
                std::io::Write::write_all(&mut encoder, json_string.as_bytes()).unwrap();
                encoder.finish().unwrap().into()
            }))
            .await
            .is_err()
        {
            debug!("Docker websocket connection closed");
            break;
        }
        if !docker_accessible {
            break;
        }
        interval.tick().await;
    }
}

// processes
pub async fn ws_handler_p(
    ws: WebSocketUpgrade,
    State((sys, config)): State<(Arc<Mutex<System>>, Config)>,
) -> impl IntoResponse {
    debug!("Processes websocket connection requested");
    ws.on_upgrade(move |socket| handle_socket_p(socket, sys, config.update_interval))
}

async fn handle_socket_p(mut socket: WebSocket, sys: Arc<Mutex<System>>, ws_interval: u64) {
    debug!("Processes websocket connection established");
    let mut interval = interval(Duration::from_secs(ws_interval));
    loop {
        let processes_info = collect_info::collect_processes_info(&sys.lock().unwrap());
        if socket
            .send(Message::Binary({
                let json_string = serde_json::to_string(&processes_info).unwrap();
                let mut encoder =
                    flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
                std::io::Write::write_all(&mut encoder, json_string.as_bytes()).unwrap();
                encoder.finish().unwrap().into()
            }))
            .await
            .is_err()
        {
            debug!("Processes websocket connection closed");
            break;
        }
        interval.tick().await;
    }
}

// general info
pub async fn ws_handler_g(
    ws: WebSocketUpgrade,
    State((sys, config)): State<(Arc<Mutex<System>>, Config)>,
) -> impl IntoResponse {
    debug!("General system info websocket connection requested");
    ws.on_upgrade(move |socket| handle_socket_g(socket, sys, config.update_interval))
}

async fn handle_socket_g(mut socket: WebSocket, sys: Arc<Mutex<System>>, ws_interval: u64) {
    debug!("General system info websocket connection established");
    let mut interval = interval(Duration::from_secs(ws_interval));
    loop {
        let general_info = collect_info::collect_general_info(&sys.lock().unwrap());
        if socket
            .send(Message::Binary({
                let json_string = serde_json::to_string(&general_info).unwrap();
                let mut encoder =
                    flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
                std::io::Write::write_all(&mut encoder, json_string.as_bytes()).unwrap();
                encoder.finish().unwrap().into()
            }))
            .await
            .is_err()
        {
            debug!("General system info websocket connection closed");
            break;
        }
        interval.tick().await;
    }
}

pub async fn get_container_logs(Path(container_id): Path<String>) -> impl IntoResponse {
    debug!("Getting logs for container: {}", container_id);
    let docker = bollard::Docker::connect_with_local_defaults().unwrap();
    let options = Some(LogsOptions::<String> {
        stdout: true,
        stderr: true,
        timestamps: true,
        ..Default::default()
    });

    let mut logs = String::new();
    // Fetch and print logs
    let mut logs_stream = docker.logs(&container_id, options);
    while let Some(log_result) = logs_stream.next().await {
        match log_result {
            Ok(log_output) => match log_output {
                bollard::container::LogOutput::StdOut { message } => {
                    logs.push_str(format!("STDOUT|{}", String::from_utf8_lossy(&message)).as_str());
                }
                bollard::container::LogOutput::StdErr { message } => {
                    logs.push_str(format!("STDERR|{}", String::from_utf8_lossy(&message)).as_str());
                }
                _ => {}
            },
            Err(e) => {
                error!("Error getting logs for container {}: {}", container_id, e);
                break;
            }
        }
    }

    Html(logs)
}

// Historical data endpoint
pub async fn historical_data(
    Query(params): Query<HistoricalQueryOptions>,
    State((_, config)): State<(Arc<Mutex<System>>, Config)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    debug!("Historical data requested: {:?}", params);
    // Open database connection
    let db = match Database::new(&config.db_path) {
        Ok(db) => db,
        Err(e) => {
            error!("Failed to open database: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to open database: {}", e),
            ));
        }
    };

    // Query historical data
    match db.query_historical_data(&params) {
        Ok(data) => {
            debug!("Historical data query successful: {} records", data.len());
            Ok(Json(data))
        }
        Err(e) => {
            error!("Failed to query database: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to query database: {}", e),
            ))
        }
    }
}

pub async fn add_notif_method(
    State((_, config)): State<(Arc<Mutex<System>>, Config)>,
    body: Result<Json<NotificationMethod>, JsonRejection>,
) -> impl IntoResponse {
    let mut notification_method = match body {
        Ok(Json(method)) => method,
        Err(err) => {
            error!("Invalid notification method JSON payload: {}", err);
            return (
                StatusCode::BAD_REQUEST,
                format!("Invalid JSON payload: {}", err),
            )
                .into_response();
        }
    };

    info!("Adding notification method: {}", notification_method.name);
    debug!("Notification method details: {:?}", notification_method);

    let db = match Database::new(&config.db_path) {
        Ok(db) => db,
        Err(e) => {
            error!("Failed to open database: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to open database: {}", e),
            )
                .into_response();
        }
    };

    let mut methods: Vec<NotificationMethod> =
        match db.get_kv_str("notification_sources").unwrap_or_default() {
            Some(methods) => serde_json::from_str(&methods).unwrap(),
            None => Vec::new(),
        };

    if notification_method.id == "-1" {
        notification_method.id = uuid::Uuid::new_v4().to_string();
        info!(
            "Created new notification method with ID: {}",
            notification_method.id
        );
    } else {
        info!(
            "Updating notification method with ID: {}",
            notification_method.id
        );
        methods.retain(|method| method.id != notification_method.id);
    }

    methods.push(notification_method);

    db.set_kv_str(
        "notification_methods",
        &serde_json::to_string(&methods).unwrap().to_string(),
    )
    .unwrap();

    (StatusCode::CREATED, Json(methods)).into_response()
}

pub async fn get_notif_methods(
    State((_, config)): State<(Arc<Mutex<System>>, Config)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let db = match Database::new(&config.db_path) {
        Ok(db) => db,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to open database: {}", e),
            ));
        }
    };

    let methods: Vec<NotificationMethod> =
        match db.get_kv_str("notification_methods").unwrap_or_default() {
            Some(methods) => serde_json::from_str(&methods).unwrap(),
            None => Vec::new(),
        };

    Ok(Json(methods))
}

pub async fn delete_notif_method(
    State((_, config)): State<(Arc<Mutex<System>>, Config)>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let db = match Database::new(&config.db_path) {
        Ok(db) => db,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to open database: {}", e),
            ));
        }
    };

    let mut methods: Vec<NotificationMethod> =
        match db.get_kv_str("notification_sources").unwrap_or_default() {
            Some(methods) => serde_json::from_str(&methods).unwrap(),
            None => Vec::new(),
        };

    methods.retain(|source| source.id != id);

    db.set_kv_str(
        "notification_methods",
        &serde_json::to_string(&methods).unwrap().to_string(),
    )
    .unwrap();

    Ok(Json(methods))
}

pub async fn add_alert(
    State((_, config)): State<(Arc<Mutex<System>>, Config)>,
    body: Result<Json<models::Alert>, JsonRejection>,
) -> impl IntoResponse {
    let mut alert = match body {
        Ok(Json(alert)) => alert,
        Err(err) => {
            error!("Invalid alert JSON payload: {}", err);
            return (
                StatusCode::BAD_REQUEST,
                format!("Invalid JSON payload: {}", err),
            )
                .into_response();
        }
    };

    info!("Adding alert for {}", alert.var.var);
    debug!("Alert details: {:?}", alert);

    let db = match Database::new(&config.db_path) {
        Ok(db) => db,
        Err(e) => {
            error!("Failed to open database: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to open database: {}", e),
            )
                .into_response();
        }
    };

    let mut alerts: Vec<models::Alert> = match db.get_kv_str("alerts").unwrap_or_default() {
        Some(alerts) => serde_json::from_str(&alerts).unwrap(),
        None => Vec::new(),
    };

    if alert.id == "-1" {
        alert.id = uuid::Uuid::new_v4().to_string();
        info!("Created new alert with ID: {}", alert.id);
    } else {
        info!("Updating alert with ID: {}", alert.id);
        alerts.retain(|a| a.id != alert.id);
    }

    alerts.push(alert);

    db.set_kv_str(
        "alerts",
        &serde_json::to_string(&alerts).unwrap().to_string(),
    )
    .unwrap();

    (StatusCode::CREATED, Json(alerts)).into_response()
}

pub async fn get_alerts(
    State((_, config)): State<(Arc<Mutex<System>>, Config)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let db = match Database::new(&config.db_path) {
        Ok(db) => db,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to open database: {}", e),
            ));
        }
    };

    let alerts: Vec<models::Alert> = match db.get_kv_str("alerts").unwrap_or_default() {
        Some(alerts) => serde_json::from_str(&alerts).unwrap(),
        None => Vec::new(),
    };

    Ok(Json(alerts))
}

pub async fn delete_alert(
    State((_, config)): State<(Arc<Mutex<System>>, Config)>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    info!("Deleting alert with ID: {}", id);

    let db = match Database::new(&config.db_path) {
        Ok(db) => db,
        Err(e) => {
            error!("Failed to open database: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to open database: {}", e),
            ));
        }
    };

    let mut alerts: Vec<models::Alert> = match db.get_kv_str("alerts").unwrap_or_default() {
        Some(alerts) => serde_json::from_str(&alerts).unwrap(),
        None => Vec::new(),
    };

    alerts.retain(|alert| alert.id != id);

    db.set_kv_str(
        "alerts",
        &serde_json::to_string(&alerts).unwrap().to_string(),
    )
    .unwrap();

    Ok(Json(alerts))
}

pub async fn get_alert_vars(
    State((_, config)): State<(Arc<Mutex<System>>, Config)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let db = match Database::new(&config.db_path) {
        Ok(db) => db,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to open database: {}", e),
            ));
        }
    };

    let vars: Vec<models::AlertVar> = match db.get_resource_list() {
        Ok(vars) => vars,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get resource list: {}", e),
            ));
        }
    };

    Ok(Json(vars))
}
