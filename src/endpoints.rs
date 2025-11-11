use crate::collect_info;
use crate::config::Config;
use crate::db::Database;
use crate::models::{self, ApiResponse, DirectoryListing, FileEntry, NotificationMethod};
use axum::Json;
use axum::body::Body;
use axum::extract::rejection::JsonRejection;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Multipart, Path, Query, Request};
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
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use sysinfo::System;
use tokio::{
    self,
    io::AsyncWriteExt,
    time::{Duration, interval},
};

#[derive(Embed)]
#[folder = "web/build/static"]
struct Asset;

/// Validates that a path is within one of the allowed serve directories
/// Returns the canonical path if allowed, or None if access should be denied
fn validate_path_access(path: &str, allowed_dirs: &[String]) -> Option<PathBuf> {
    let path_buf = PathBuf::from(path);
    let canonical_path = path_buf.canonicalize().ok()?;

    for serve_dir in allowed_dirs {
        if let Ok(serve_path) = PathBuf::from(serve_dir).canonicalize()
            && canonical_path.starts_with(&serve_path)
        {
            return Some(canonical_path);
        }
    }

    None
}

/// Generates a unique path by appending a number if the file/folder already exists
/// e.g., "file.txt" -> "file (1).txt" -> "file (2).txt"
fn generate_unique_path(path: &std::path::Path) -> PathBuf {
    if !path.exists() {
        return path.to_path_buf();
    }

    if path.file_name().is_none() || path.file_name().unwrap().to_str().is_none() {
        return path.to_path_buf();
    }

    let parent = path.parent();
    let file_name = path.file_name().unwrap().to_str().unwrap();

    // Split filename and extension
    let (name, ext) = if let Some(dot_pos) = file_name.rfind('.') {
        let name = &file_name[..dot_pos];
        let ext = &file_name[dot_pos..];
        (name, ext)
    } else {
        (file_name, "")
    };

    // Try to find a unique name
    for i in 1..100 {
        let new_name = if ext.is_empty() {
            format!("{} ({})", name, i)
        } else {
            format!("{} ({}){}", name, i, ext)
        };

        let new_path = if let Some(p) = parent {
            p.join(new_name)
        } else {
            PathBuf::from(new_name)
        };

        if !new_path.exists() {
            return new_path;
        }
    }

    // Fallback: use timestamp
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let new_name = if ext.is_empty() {
        format!("{} ({})", name, timestamp)
    } else {
        format!("{} ({}){}", name, timestamp, ext)
    };

    if let Some(p) = parent {
        p.join(new_name)
    } else {
        PathBuf::from(new_name)
    }
}

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

    match Asset::get(path) {
        Some(content) => {
            let etag = hex::encode(content.metadata.sha256_hash());

            // Check If-None-Match header
            if let Some(if_none_match) = request.headers().get("If-None-Match")
                && let Ok(if_none_match_str) = if_none_match.to_str()
                && if_none_match_str == etag
            {
                return StatusCode::NOT_MODIFIED.into_response();
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

pub async fn fallback_handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    request: Request<Body>,
) -> axum::response::Response {
    let path = request.uri().path();

    if path.ends_with("reqinfo") {
        return req_info(ConnectInfo(addr), headers, request)
            .await
            .into_response();
    }

    // Otherwise, serve static files
    serve_static(request).await.into_response()
}

pub async fn req_info(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    request: Request<Body>,
) -> String {
    let method = request.method().to_string();
    let path = request.uri().path().to_string();

    let headers_str = headers
        .iter()
        .map(|(k, v)| format!("{}: {}", k, v.to_str().unwrap()))
        .collect::<Vec<String>>()
        .join("\n");

    info!("Request info from IP: {}", addr);
    debug!("Headers: {}", headers_str);

    format!(
        "Method: {}\nPath: {}\nIP: {}\n\nHeaders:\n{}",
        method, path, addr, headers_str
    )
}

pub async fn capabilities_handler(
    State((_, config)): State<(Arc<Mutex<System>>, Arc<Config>)>,
) -> impl IntoResponse {
    debug!("Capabilities requested");
    Json(ApiResponse::success(config.system_capabilities.clone())).into_response()
}

// docker
pub async fn ws_handler_d(
    ws: WebSocketUpgrade,
    State((_, config)): State<(Arc<Mutex<System>>, Arc<Config>)>,
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
    State((sys, config)): State<(Arc<Mutex<System>>, Arc<Config>)>,
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
    State((sys, config)): State<(Arc<Mutex<System>>, Arc<Config>)>,
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
    State((_, config)): State<(Arc<Mutex<System>>, Arc<Config>)>,
) -> impl IntoResponse {
    debug!("Historical data requested: {:?}", params);
    // Open database connection
    let db = match Database::new(&config.db_path) {
        Ok(db) => db,
        Err(e) => {
            error!("Failed to open database: {}", e);
            return Json(ApiResponse::<Vec<models::HistoricalSeries>>::error(
                format!("Failed to open database: {}", e),
            ))
            .into_response();
        }
    };

    // Query historical data
    match db.query_historical_data(&params) {
        Ok(data) => {
            debug!("Historical data query successful: {} records", data.len());
            Json(ApiResponse::success(data)).into_response()
        }
        Err(e) => {
            error!("Failed to query database: {}", e);
            Json(ApiResponse::<Vec<models::HistoricalSeries>>::error(
                format!("Failed to query database: {}", e),
            ))
            .into_response()
        }
    }
}

pub async fn add_notif_method(
    State((_, config)): State<(Arc<Mutex<System>>, Arc<Config>)>,
    body: Result<Json<NotificationMethod>, JsonRejection>,
) -> impl IntoResponse {
    let mut notification_method = match body {
        Ok(Json(method)) => method,
        Err(err) => {
            error!("Invalid notification method JSON payload: {}", err);
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::error(format!(
                    "Invalid JSON payload: {}",
                    err
                ))),
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
                Json(ApiResponse::<()>::error(format!(
                    "Failed to open database: {}",
                    e
                ))),
            )
                .into_response();
        }
    };

    let mut methods: Vec<NotificationMethod> =
        match db.get_kv_str("notification_methods").unwrap_or_default() {
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

    (StatusCode::CREATED, Json(ApiResponse::success(methods))).into_response()
}

pub async fn get_notif_methods(
    State((_, config)): State<(Arc<Mutex<System>>, Arc<Config>)>,
) -> impl IntoResponse {
    let db = match Database::new(&config.db_path) {
        Ok(db) => db,
        Err(e) => {
            return Json(ApiResponse::<Vec<NotificationMethod>>::error(format!(
                "Failed to open database: {}",
                e
            )))
            .into_response();
        }
    };

    let methods: Vec<NotificationMethod> =
        match db.get_kv_str("notification_methods").unwrap_or_default() {
            Some(methods) => serde_json::from_str(&methods).unwrap(),
            None => Vec::new(),
        };

    Json(ApiResponse::success(methods)).into_response()
}

pub async fn delete_notif_method(
    State((_, config)): State<(Arc<Mutex<System>>, Arc<Config>)>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let db = match Database::new(&config.db_path) {
        Ok(db) => db,
        Err(e) => {
            return Json(ApiResponse::<Vec<NotificationMethod>>::error(format!(
                "Failed to open database: {}",
                e
            )))
            .into_response();
        }
    };

    let mut methods: Vec<NotificationMethod> =
        match db.get_kv_str("notification_methods").unwrap_or_default() {
            Some(methods) => serde_json::from_str(&methods).unwrap(),
            None => Vec::new(),
        };

    methods.retain(|source| source.id != id);

    db.set_kv_str(
        "notification_methods",
        &serde_json::to_string(&methods).unwrap().to_string(),
    )
    .unwrap();

    Json(ApiResponse::success(methods)).into_response()
}

pub async fn add_alert(
    State((_, config)): State<(Arc<Mutex<System>>, Arc<Config>)>,
    body: Result<Json<models::Alert>, JsonRejection>,
) -> impl IntoResponse {
    let mut alert = match body {
        Ok(Json(alert)) => alert,
        Err(err) => {
            error!("Invalid alert JSON payload: {}", err);
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::error(format!(
                    "Invalid JSON payload: {}",
                    err
                ))),
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
                Json(ApiResponse::<()>::error(format!(
                    "Failed to open database: {}",
                    e
                ))),
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

    (StatusCode::CREATED, Json(ApiResponse::success(alerts))).into_response()
}

pub async fn get_alerts(
    State((_, config)): State<(Arc<Mutex<System>>, Arc<Config>)>,
) -> impl IntoResponse {
    let db = match Database::new(&config.db_path) {
        Ok(db) => db,
        Err(e) => {
            return Json(ApiResponse::<Vec<models::Alert>>::error(format!(
                "Failed to open database: {}",
                e
            )))
            .into_response();
        }
    };

    let alerts: Vec<models::Alert> = match db.get_kv_str("alerts").unwrap_or_default() {
        Some(alerts) => serde_json::from_str(&alerts).unwrap(),
        None => Vec::new(),
    };

    Json(ApiResponse::success(alerts)).into_response()
}

pub async fn delete_alert(
    State((_, config)): State<(Arc<Mutex<System>>, Arc<Config>)>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    info!("Deleting alert with ID: {}", id);

    let db = match Database::new(&config.db_path) {
        Ok(db) => db,
        Err(e) => {
            error!("Failed to open database: {}", e);
            return Json(ApiResponse::<Vec<models::Alert>>::error(format!(
                "Failed to open database: {}",
                e
            )))
            .into_response();
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

    Json(ApiResponse::success(alerts)).into_response()
}

pub async fn get_alert_vars(
    State((_, config)): State<(Arc<Mutex<System>>, Arc<Config>)>,
) -> impl IntoResponse {
    let db = match Database::new(&config.db_path) {
        Ok(db) => db,
        Err(e) => {
            return Json(ApiResponse::<Vec<models::AlertVar>>::error(format!(
                "Failed to open database: {}",
                e
            )))
            .into_response();
        }
    };

    let vars: Vec<models::AlertVar> = match db.get_resource_list() {
        Ok(vars) => vars,
        Err(e) => {
            return Json(ApiResponse::<Vec<models::AlertVar>>::error(format!(
                "Failed to get resource list: {}",
                e
            )))
            .into_response();
        }
    };

    Json(ApiResponse::success(vars)).into_response()
}

pub async fn get_serve_dirs(
    State((_, config)): State<(Arc<Mutex<System>>, Arc<Config>)>,
) -> impl IntoResponse {
    debug!("Getting serve directories");
    if !config.system_capabilities.file_serving {
        return Json(ApiResponse::<Vec<String>>::error(
            "File serving is disabled".to_string(),
        ))
        .into_response();
    }
    Json(ApiResponse::success(config.serve_dirs.clone())).into_response()
}

pub async fn browse_directory(
    Query(params): Query<std::collections::HashMap<String, String>>,
    State((_, config)): State<(Arc<Mutex<System>>, Arc<Config>)>,
) -> impl IntoResponse {
    if !config.system_capabilities.file_serving {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<String>::error(
                "File serving is disabled".to_string(),
            )),
        )
            .into_response();
    }

    let path = match params.get("path") {
        Some(p) => p,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<DirectoryListing>::error(
                    "Missing path parameter".to_string(),
                )),
            )
                .into_response();
        }
    };

    debug!("Browsing directory: {}", path);

    // Security check: Ensure the path is within one of the allowed serve_dirs
    let canonical_path = match validate_path_access(path, &config.serve_dirs) {
        Some(p) => p,
        None => {
            warn!("Access denied to path: {}", path);
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::<DirectoryListing>::error(
                    "Access denied".to_string(),
                )),
            )
                .into_response();
        }
    };

    // Read directory contents
    let entries = match fs::read_dir(&canonical_path) {
        Ok(entries) => entries,
        Err(e) => {
            error!("Failed to read directory: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<DirectoryListing>::error(format!(
                    "Failed to read directory: {}",
                    e
                ))),
            )
                .into_response();
        }
    };

    let mut file_entries: Vec<FileEntry> = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let name = entry.file_name().to_string_lossy().to_string();
        let is_dir = metadata.is_dir();
        let size = metadata.len();
        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // Get permissions string (Unix-style)
        #[cfg(unix)]
        let permissions = {
            use std::os::unix::fs::PermissionsExt;
            let mode = metadata.permissions().mode();
            format!(
                "{}{}{}{}{}{}{}{}{}",
                if mode & 0o400 != 0 { 'r' } else { '-' },
                if mode & 0o200 != 0 { 'w' } else { '-' },
                if mode & 0o100 != 0 { 'x' } else { '-' },
                if mode & 0o040 != 0 { 'r' } else { '-' },
                if mode & 0o020 != 0 { 'w' } else { '-' },
                if mode & 0o010 != 0 { 'x' } else { '-' },
                if mode & 0o004 != 0 { 'r' } else { '-' },
                if mode & 0o002 != 0 { 'w' } else { '-' },
                if mode & 0o001 != 0 { 'x' } else { '-' },
            )
        };

        #[cfg(not(unix))]
        let permissions = if metadata.permissions().readonly() {
            "r--r--r--".to_string()
        } else {
            "rw-rw-rw-".to_string()
        };

        file_entries.push(FileEntry {
            name,
            is_dir,
            size,
            modified,
            permissions,
        });
    }

    // Sort: directories first, then by name
    file_entries.sort_by(|a, b| {
        if a.is_dir == b.is_dir {
            a.name.to_lowercase().cmp(&b.name.to_lowercase())
        } else if a.is_dir {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });

    Json(ApiResponse::success(DirectoryListing {
        path: path.clone(),
        entries: file_entries,
    }))
    .into_response()
}

pub async fn download_file(
    Query(params): Query<std::collections::HashMap<String, String>>,
    State((_, config)): State<(Arc<Mutex<System>>, Arc<Config>)>,
) -> impl IntoResponse {
    if !config.system_capabilities.file_serving {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<String>::error(
                "File serving is disabled".to_string(),
            )),
        )
            .into_response();
    }

    let path = match params.get("path") {
        Some(p) => p,
        None => {
            return (StatusCode::BAD_REQUEST, "Missing path parameter").into_response();
        }
    };

    // Check if this is a view request (inline) or download request (attachment)
    let is_inline = params.get("inline").map(|v| v == "true").unwrap_or(false);

    if is_inline {
        debug!("Viewing file inline: {}", path);
    } else {
        debug!("Downloading file: {}", path);
    }

    // Security check: Ensure the path is within one of the allowed serve_dirs
    let canonical_path = match validate_path_access(path, &config.serve_dirs) {
        Some(p) => p,
        None => {
            warn!("Access denied to file: {}", path);
            return (StatusCode::FORBIDDEN, "Access denied").into_response();
        }
    };

    // Check if it's a file (not a directory)
    if canonical_path.is_dir() {
        return (StatusCode::BAD_REQUEST, "Cannot download a directory").into_response();
    }

    // Read file contents
    match tokio::fs::read(&canonical_path).await {
        Ok(contents) => {
            let filename = canonical_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("download");

            let mime_type = mime_guess::from_path(&canonical_path)
                .first_or_octet_stream()
                .to_string();

            let content_disposition = if is_inline {
                format!("inline; filename=\"{}\"", filename)
            } else {
                format!("attachment; filename=\"{}\"", filename)
            };

            (
                StatusCode::OK,
                [
                    ("Content-Type", mime_type.as_str()),
                    ("Content-Disposition", content_disposition.as_str()),
                ],
                contents,
            )
                .into_response()
        }
        Err(e) => {
            error!("Failed to read file: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read file: {}", e),
            )
                .into_response()
        }
    }
}

pub async fn get_file_content(
    Query(params): Query<std::collections::HashMap<String, String>>,
    State((_, config)): State<(Arc<Mutex<System>>, Arc<Config>)>,
) -> impl IntoResponse {
    if !config.system_capabilities.file_serving {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<String>::error(
                "File serving is disabled".to_string(),
            )),
        )
            .into_response();
    }

    let path = match params.get("path") {
        Some(p) => p,
        None => {
            return (StatusCode::BAD_REQUEST, "Missing path parameter").into_response();
        }
    };

    debug!("Getting file content: {}", path);

    // Security check: Ensure the path is within one of the allowed serve_dirs
    let canonical_path = match validate_path_access(path, &config.serve_dirs) {
        Some(p) => p,
        None => {
            warn!("Access denied to file: {}", path);
            return (StatusCode::FORBIDDEN, "Access denied").into_response();
        }
    };

    // Check if it's a file (not a directory)
    if canonical_path.is_dir() {
        return (StatusCode::BAD_REQUEST, "Cannot read a directory").into_response();
    }

    // Check file size (limit to 10MB for text viewing)
    match tokio::fs::metadata(&canonical_path).await {
        Ok(metadata) => {
            if metadata.len() > 10_485_760 {
                // 10MB
                return (StatusCode::PAYLOAD_TOO_LARGE, "File too large to display")
                    .into_response();
            }
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to read file metadata",
            )
                .into_response();
        }
    }

    // Read file contents
    match tokio::fs::read_to_string(&canonical_path).await {
        Ok(contents) => {
            let mime_type = mime_guess::from_path(&canonical_path)
                .first_or_octet_stream()
                .to_string();

            (
                StatusCode::OK,
                [("Content-Type", mime_type.as_str())],
                contents,
            )
                .into_response()
        }
        Err(e) => {
            error!("Failed to read file: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read file: {}", e),
            )
                .into_response()
        }
    }
}

pub async fn upload_file(
    State((_, config)): State<(Arc<Mutex<System>>, Arc<Config>)>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    if !config.system_capabilities.file_serving {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<String>::error(
                "File serving is disabled".to_string(),
            )),
        )
            .into_response();
    }

    let mut upload_path: Option<String> = None;
    let mut uploaded_files = Vec::new();
    let mut errors = Vec::new();
    let mut file_count = 0;

    // Parse multipart fields
    loop {
        let field = match multipart.next_field().await {
            Ok(Some(f)) => f,
            Ok(None) => break,
            Err(e) => {
                error!("Failed to read multipart field: {}", e);
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<String>::error(format!(
                        "Failed to read multipart data: {}",
                        e
                    ))),
                )
                    .into_response();
            }
        };

        let field_name = field.name().unwrap_or("").to_string();

        if field_name == "path" {
            match field.text().await {
                Ok(text) => upload_path = Some(text),
                Err(e) => {
                    error!("Failed to read path field: {}", e);
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(ApiResponse::<String>::error(format!(
                            "Failed to read path: {}",
                            e
                        ))),
                    )
                        .into_response();
                }
            }
        } else if field_name == "file" || field_name.starts_with("file-") {
            // We need the upload path before processing files
            let path = match &upload_path {
                Some(p) => p,
                None => {
                    error!("Received file before path parameter");
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(ApiResponse::<String>::error(
                            "Path parameter must be sent before files".to_string(),
                        )),
                    )
                        .into_response();
                }
            };

            // Security check: Ensure the base path is within one of the allowed serve_dirs
            let canonical_base_path = match validate_path_access(path, &config.serve_dirs) {
                Some(p) => p,
                None => {
                    warn!("Access denied to path: {}", path);
                    return (
                        StatusCode::FORBIDDEN,
                        Json(ApiResponse::<String>::error("Access denied".to_string())),
                    )
                        .into_response();
                }
            };

            // Ensure the target path is a directory
            if !canonical_base_path.is_dir() {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<String>::error(
                        "Target path is not a directory".to_string(),
                    )),
                )
                    .into_response();
            }

            // Get the file name - this can be the full path for folder uploads
            let file_name = match field.file_name() {
                Some(name) => name,
                None => {
                    error!("Missing file name in upload");
                    errors.push("Unnamed file: Missing file name".to_string());
                    continue;
                }
            };

            file_count += 1;

            debug!("Processing file: {}", file_name);

            // Clean up the relative path (remove leading slashes, etc.)
            // Canonicalize the path to prevent directory traversal
            let mut canonical_file_name = PathBuf::from(file_name.trim_start_matches('/'));
            match canonical_file_name.canonicalize() {
                Ok(p) => {
                    canonical_file_name = p;
                }
                Err(_) => {
                    // return an error if the path cannot be canonicalized
                    error!("Invalid file path: {}", file_name);
                    errors.push(format!("{}: Invalid file path", file_name));
                    continue;
                }
            }

            // Construct the full file path
            let mut file_path = match canonical_base_path.join(canonical_file_name).canonicalize() {
                Ok(p) => p,
                Err(_) => {
                    // return an error if the path cannot be canonicalized
                    error!("Invalid file path after join: {}", file_name);
                    errors.push(format!("{}: Invalid file path", file_name));
                    continue;
                }
            };

            // Check if file exists and rename if necessary
            if file_path.exists() {
                file_path = generate_unique_path(&file_path);
                debug!("File exists, renamed to: {:?}", file_path);
            }

            // Validate the final path is still within allowed directories (if file_path is ../sth)
            let file_path_str = file_path.to_str().unwrap();
            if validate_path_access(file_path_str, &config.serve_dirs).is_none() {
                warn!("Access denied to final path: {:?}", file_path);
                errors.push(format!("{}: Access denied", file_path_str));
                continue;
            }

            // Create parent directories if they don't exist (for folder uploads)
            if let Some(parent) = file_path.parent()
                && !parent.exists()
                && let Err(e) = tokio::fs::create_dir_all(parent).await
            {
                error!("Failed to create directory {:?}: {}", parent, e);
                errors.push(format!(
                    "{}: Failed to create directory",
                    parent.to_str().unwrap()
                ));
                continue;
            }

            // Stream the file directly to disk
            let file_result: Result<u64, Box<dyn std::error::Error>> = async {
                let mut file = tokio::fs::File::create(&file_path).await?;
                let mut stream = field;
                let mut total_bytes = 0u64;

                while let Some(chunk) = stream.chunk().await? {
                    file.write_all(&chunk).await?;
                    total_bytes += chunk.len() as u64;
                }

                file.flush().await?;
                Ok(total_bytes)
            }
            .await;

            match file_result {
                Ok(bytes_written) => {
                    let uploaded_name = file_path
                        .strip_prefix(&canonical_base_path)
                        .unwrap_or(&file_path)
                        .to_string_lossy()
                        .to_string();

                    info!(
                        "File uploaded successfully: {:?} ({} bytes)",
                        file_path, bytes_written
                    );
                    uploaded_files.push(uploaded_name);
                }
                Err(e) => {
                    error!("Failed to write file {:?}: {}", file_path, e);
                    errors.push(format!("{}: {}", file_path_str, e));
                }
            }
        }
    }

    // Verify we got a path
    if upload_path.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<String>::error(
                "Missing path parameter".to_string(),
            )),
        )
            .into_response();
    }

    if file_count == 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<String>::error(
                "No files provided".to_string(),
            )),
        )
            .into_response();
    }

    // Build response message
    let message = if errors.is_empty() {
        if uploaded_files.len() == 1 {
            format!("File '{}' uploaded successfully", uploaded_files[0])
        } else {
            format!("{} files uploaded successfully", uploaded_files.len())
        }
    } else if uploaded_files.is_empty() {
        format!("Failed to upload files: {}", errors.join(", "))
    } else {
        format!(
            "{} files uploaded, {} failed: {}",
            uploaded_files.len(),
            errors.len(),
            errors.join(", ")
        )
    };

    let status = if uploaded_files.is_empty() {
        StatusCode::INTERNAL_SERVER_ERROR
    } else {
        StatusCode::OK
    };

    (status, Json(ApiResponse::success(message))).into_response()
}

pub async fn create_folder(
    Query(params): Query<std::collections::HashMap<String, String>>,
    State((_, config)): State<(Arc<Mutex<System>>, Arc<Config>)>,
) -> impl IntoResponse {
    if !config.system_capabilities.file_serving {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<String>::error(
                "File serving is disabled".to_string(),
            )),
        )
            .into_response();
    }

    let path = match params.get("path") {
        Some(p) => p,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<String>::error(
                    "Missing path parameter".to_string(),
                )),
            )
                .into_response();
        }
    };

    let folder_name = match params.get("name") {
        Some(n) => n,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<String>::error(
                    "Missing name parameter".to_string(),
                )),
            )
                .into_response();
        }
    };

    debug!("Creating folder: {} in {}", folder_name, path);

    // Security check: Ensure the path is within one of the allowed serve_dirs
    let canonical_path = match validate_path_access(path, &config.serve_dirs) {
        Some(p) => p,
        None => {
            warn!("Access denied to path: {}", path);
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::<String>::error("Access denied".to_string())),
            )
                .into_response();
        }
    };

    // Ensure the target path is a directory
    if !canonical_path.is_dir() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<String>::error(
                "Target path is not a directory".to_string(),
            )),
        )
            .into_response();
    }

    // Construct the folder path
    let canonical_folder_name = match PathBuf::from(folder_name).canonicalize() {
        Ok(p) => p,
        Err(_) => {
            // return an error if the path cannot be canonicalized
            error!("Invalid folder name: {}", folder_name);
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<String>::error(
                    "Invalid folder name".to_string(),
                )),
            )
                .into_response();
        }
    };
    let mut folder_path = canonical_path.join(canonical_folder_name.clone());

    // Validate the final path is still within allowed directories
    let folder_path_str = folder_path.to_str().unwrap();
    if validate_path_access(folder_path_str, &config.serve_dirs).is_none() {
        warn!("Access denied to final path: {:?}", folder_path);
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<String>::error("Access denied".to_string())),
        )
            .into_response();
    }

    // Check if folder exists and rename if necessary
    if folder_path.exists() {
        folder_path = generate_unique_path(&folder_path);
    }

    // Create the folder
    match tokio::fs::create_dir(&folder_path).await {
        Ok(_) => {
            info!("Folder created successfully: {:?}", folder_path);
            (
                StatusCode::OK,
                Json(ApiResponse::success(format!(
                    "Folder '{}' created successfully",
                    canonical_folder_name.to_str().unwrap_or("unknown")
                ))),
            )
                .into_response()
        }
        Err(e) => {
            error!("Failed to create folder: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<String>::error(format!(
                    "Failed to create folder: {}",
                    e
                ))),
            )
                .into_response()
        }
    }
}

pub async fn move_file(
    Query(params): Query<std::collections::HashMap<String, String>>,
    State((_, config)): State<(Arc<Mutex<System>>, Arc<Config>)>,
) -> impl IntoResponse {
    if !config.system_capabilities.file_serving {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<String>::error(
                "File serving is disabled".to_string(),
            )),
        )
            .into_response();
    }

    let source = match params.get("source") {
        Some(p) => p,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<String>::error(
                    "Missing source parameter".to_string(),
                )),
            )
                .into_response();
        }
    };

    let destination = match params.get("destination") {
        Some(d) => d,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<String>::error(
                    "Missing destination parameter".to_string(),
                )),
            )
                .into_response();
        }
    };

    debug!("Moving: {} to {}", source, destination);

    // Security check: Ensure the source is within one of the allowed serve_dirs
    let canonical_source = match validate_path_access(source, &config.serve_dirs) {
        Some(p) => p,
        None => {
            warn!("Access denied to path: {}", source);
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::<String>::error("Access denied".to_string())),
            )
                .into_response();
        }
    };

    // Security check: Ensure the destination is within one of the allowed serve_dirs
    let canonical_destination = match validate_path_access(destination, &config.serve_dirs) {
        Some(p) => p,
        None => {
            warn!("Access denied to path: {}", destination);
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::<String>::error("Access denied".to_string())),
            )
                .into_response();
        }
    };

    // Check if target already exists
    if canonical_destination.exists() {
        let name = canonical_destination
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        return (
            StatusCode::CONFLICT,
            Json(ApiResponse::<String>::error(format!(
                "A file or folder named '{}' already exists at destination",
                name
            ))),
        )
            .into_response();
    }

    // Move/rename the file/folder
    match tokio::fs::rename(&canonical_source, &canonical_destination).await {
        Ok(_) => {
            info!(
                "Moved successfully: {:?} -> {:?}",
                canonical_source, canonical_destination
            );
            let name = canonical_destination
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            (
                StatusCode::OK,
                Json(ApiResponse::success(format!(
                    "Moved to '{}' successfully",
                    name
                ))),
            )
                .into_response()
        }
        Err(e) => {
            error!("Failed to move: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<String>::error(format!(
                    "Failed to move: {}",
                    e
                ))),
            )
                .into_response()
        }
    }
}

pub async fn delete_file(
    Query(params): Query<std::collections::HashMap<String, String>>,
    State((_, config)): State<(Arc<Mutex<System>>, Arc<Config>)>,
) -> impl IntoResponse {
    if !config.system_capabilities.file_serving {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<String>::error(
                "File serving is disabled".to_string(),
            )),
        )
            .into_response();
    }

    let path = match params.get("path") {
        Some(p) => p,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<String>::error(
                    "Missing path parameter".to_string(),
                )),
            )
                .into_response();
        }
    };

    debug!("Deleting: {}", path);

    // Security check: Ensure the path is within one of the allowed serve_dirs
    let canonical_path = match validate_path_access(path, &config.serve_dirs) {
        Some(p) => p,
        None => {
            warn!("Access denied to path: {}", path);
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::<String>::error("Access denied".to_string())),
            )
                .into_response();
        }
    };

    let is_dir = canonical_path.is_dir();
    let name = canonical_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    // Delete the file or folder
    let result = if is_dir {
        tokio::fs::remove_dir_all(&canonical_path).await
    } else {
        tokio::fs::remove_file(&canonical_path).await
    };

    match result {
        Ok(_) => {
            info!("Deleted successfully: {:?}", canonical_path);
            (
                StatusCode::OK,
                Json(ApiResponse::success(format!(
                    "{} '{}' deleted successfully",
                    if is_dir { "Folder" } else { "File" },
                    name
                ))),
            )
                .into_response()
        }
        Err(e) => {
            error!("Failed to delete: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<String>::error(format!(
                    "Failed to delete: {}",
                    e
                ))),
            )
                .into_response()
        }
    }
}
