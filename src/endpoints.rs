use crate::collect_info;
use crate::config::Config;
use crate::db::Database;
use crate::models::{self, NotificationMethod};
use axum::Json;
use axum::body::Body;
use axum::extract::rejection::JsonRejection;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Path, Query, Request};
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
use rust_embed::Embed;
use serde_json;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use sysinfo::System;
use tokio::{
    self,
    time::{Duration, interval},
};

#[derive(Embed)]
#[folder = "web/build/static"]
struct Asset;

pub async fn serve_static(request: Request<Body>) -> impl IntoResponse {
    let mut path = request.uri().path();
    let cache_control: &str;
    if path.ends_with("favicon.png") {
        path = "favicon.png";
        cache_control = "private, max-age=7200";
    } else if path.ends_with("Inter-Regular.woff") {
        path = "Inter-Regular.woff";
        cache_control = "public, max-age=15552000, immutable";
    } else if path.ends_with("Inter-Regular.woff2") {
        path = "Inter-Regular.woff2";
        cache_control = "public, max-age=15552000, immutable";
    } else if path.ends_with("RobotoMono-Regular.woff") {
        path = "RobotoMono-Regular.woff";
        cache_control = "public, max-age=15552000, immutable";
    } else if path.ends_with("RobotoMono-Regular.woff2") {
        path = "RobotoMono-Regular.woff2";
        cache_control = "public, max-age=15552000, immutable";
    } else if path.ends_with("auth") {
        path = "auth.html";
        cache_control = "private, max-age=3600";
    } else {
        path = "index.html";
        cache_control = "private, max-age=3600";
    }

    match Asset::get(&path) {
        Some(content) => {
            let etag = hex::encode(content.metadata.sha256_hash());

            // Check If-None-Match header
            if let Some(if_none_match) = request.headers().get("If-None-Match") {
                if let Ok(if_none_match_str) = if_none_match.to_str() {
                    if if_none_match_str == etag {
                        return StatusCode::NOT_MODIFIED.into_response();
                    }
                }
            }

            (
                [
                    (
                        "Content-Type",
                        mime_guess::from_path(path)
                            .first_or_octet_stream()
                            .essence_str(),
                    ),
                    ("ETag", etag.as_str()),
                    ("Cache-Control", cache_control),
                ],
                content.data.into_response(),
            )
                .into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
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
        tail: "2000".to_string(),
        ..Default::default()
    });

    let mut logs = String::new();
    const MAX_SIZE: usize = 2097152; // Limit logs to 2 MB max

    let mut logs_stream = docker.logs(&container_id, options);
    while let Some(log_result) = logs_stream.next().await {
        match log_result {
            Ok(log_output) => match log_output {
                bollard::container::LogOutput::StdOut { message } => {
                    logs.push_str(format!("O|{}", String::from_utf8_lossy(&message)).as_str());
                }
                bollard::container::LogOutput::StdErr { message } => {
                    logs.push_str(format!("E|{}", String::from_utf8_lossy(&message)).as_str());
                }
                _ => {}
            },
            Err(e) => {
                error!("Error getting logs for container {}: {}", container_id, e);
                break;
            }
        }
        if logs.len() > MAX_SIZE {
            break;
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
    alert.firing = false;

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
