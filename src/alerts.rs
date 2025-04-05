use crate::db::Database;
use crate::models::{Alert, AlertVar, NotificationConfig, NotificationMethod, WebHookNotif};
use log::{debug, error, info, trace};
use std::collections::HashMap;
use rusqlite::params;
use tokio::time::{interval, Duration};
use reqwest::Client;
use std::sync::Arc;

/// Main function to check alerts and send notifications
pub async fn check_alerts(db_path: &str) {
    let db = match Database::new(db_path) {
        Ok(db) => Arc::new(db),
        Err(e) => {
            error!("Failed to open database: {}", e);
            return;
        }
    };

    // Run alert checking loop
    let mut interval_timer = interval(Duration::from_secs(60)); // Check every minute
    tokio::time::sleep(Duration::from_secs(70)).await;
    loop {
        debug!("Checking alerts");
        interval_timer.tick().await;
        
        let alerts = match get_alerts(&db) {
            Ok(alerts) => alerts,
            Err(e) => {
                error!("Failed to get alerts: {}", e);
                continue;
            }
        };

        if alerts.is_empty() {
            continue;
        }

        let notification_methods = match get_notification_methods(&db) {
            Ok(methods) => methods,
            Err(e) => {
                error!("Failed to get notification methods: {}", e);
                continue;
            }
        };

        let method_map: HashMap<String, NotificationMethod> = notification_methods
            .into_iter()
            .map(|m| (m.id.clone(), m))
            .collect();

        // Process each alert
        for mut alert in alerts {
            trace!("Checking alert: {:?}", alert);
            if !alert.enabled {
                continue;
            }

            // Check if alert condition is met
            let is_firing = match check_alert_condition(&db, &alert) {
                Ok(firing) => firing,
                Err(e) => {
                    error!("Failed to check alert condition for {:?}: {}", alert, e);
                    continue;
                }
            };
            trace!("Alert firing: {}", is_firing);
            
            // If alert state has changed, update it in the database
            if is_firing != alert.firing {
                alert.firing = is_firing;
                if let Err(e) = update_alert_state(&db, &alert) {
                    error!("Failed to update alert state: {}", e);
                }

                if is_firing {
                    info!("Alert fired: {:?}", alert);
                    let notification_message = format_alert_message(&alert, true);
                    
                    // Send notifications to all configured methods for this alert
                    for method_id in &alert.notif_methods {
                        if let Some(method) = method_map.get(method_id) {
                            if method.enabled {
                                if let Err(e) = send_notification(method, &notification_message).await {
                                    error!("Failed to send notification: {}", e);
                                }
                            }
                        }
                    }
                }
                else {
                    // send the relief
                    info!("Alert relief: {:?}", alert);
                    let notification_message = format_alert_message(&alert, false);

                    // Send notifications to all configured methods for this alert
                    for method_id in &alert.notif_methods {
                        if let Some(method) = method_map.get(method_id) {
                            if method.enabled {
                                if let Err(e) = send_notification(method, &notification_message).await {
                                    error!("Failed to send notification: {}", e);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn get_alerts(db: &Database) -> Result<Vec<Alert>, String> {
    let alerts_json = match db.get_kv_str("alerts") {
        Ok(Some(json)) => json,
        Ok(None) => return Ok(Vec::new()),
        Err(e) => return Err(format!("Database error: {}", e)),
    };

    serde_json::from_str::<Vec<Alert>>(&alerts_json)
        .map_err(|e| format!("Failed to parse alerts JSON: {}", e))
}

fn get_notification_methods(db: &Database) -> Result<Vec<NotificationMethod>, String> {
    let methods_json = match db.get_kv_str("notification_methods") {
        Ok(Some(json)) => json,
        Ok(None) => return Ok(Vec::new()),
        Err(e) => return Err(format!("Database error: {}", e)),
    };

    serde_json::from_str::<Vec<NotificationMethod>>(&methods_json)
        .map_err(|e| format!("Failed to parse notification methods JSON: {}", e))
}

/// Check if an alert condition is met consistently across the entire time window
fn check_alert_condition(db: &Database, alert: &Alert) -> Result<bool, String> {
    let time_window_secs = alert.time_window * 60; 
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("Time error: {}", e))?
        .as_secs();
    
    let start_time = now - time_window_secs as u64;
    
    // Choose the appropriate table based on the time window
    let table_suffix = if time_window_secs <= 7200 {
        "m" // Use minute-level data for windows <= 2 hours
    } else {
        "h" // Use hour-level data for longer windows
    };
    
    // Build query
    let conn = db.conn.lock().unwrap();
    
    let (agg_function, comparison_op) = match alert.operator.as_str() {
        ">" => ("MIN", ">"), // If MIN value > threshold, then ALL values > threshold
        "<" => ("MAX", "<"), // If MAX value < threshold, then ALL values < threshold
        _ => return Err(format!("Unknown operator: {}", alert.operator)),
    };
    
    let query_result = match alert.var.cat.as_str() {
        "sys" => {
            // System metrics are in general_* tables
            let query = format!(
                "SELECT {}({}) FROM general_{} WHERE timestamp >= ?",
                agg_function, alert.var.var, table_suffix
            );
            conn.query_row(&query, params![start_time], |row| row.get::<_, f64>(0))
        },
        "net" | "disk" => {
            // Network or disk metrics need to filter by resource name
            let query = format!(
                "SELECT {}({}) FROM {}_{} WHERE timestamp >= ? AND name = ?",
                agg_function, alert.var.var, alert.var.cat, table_suffix
            );
            conn.query_row(&query, params![start_time, alert.var.resrc], |row| row.get::<_, f64>(0))
        },
        _ => return Err(format!("Unknown category: {}", alert.var.cat)),
    };
    
    let agg_value = match query_result {
        Ok(value) => value,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            // No data in the time window
            return Ok(false); // firing=false if there's no data
        },
        Err(e) => return Err(format!("Database query error: {}", e)),
    };
    
    match comparison_op {
        ">" => Ok(agg_value > alert.threshold),
        "<" => Ok(agg_value < alert.threshold),
        _ => Err(format!("Unknown comparison operator: {}", comparison_op)),
    }
}

/// Update alert state in the database
fn update_alert_state(db: &Database, alert: &Alert) -> Result<(), String> {
    // Get current alerts
    let alerts_json = match db.get_kv_str("alerts") {
        Ok(Some(json)) => json,
        Ok(None) => "[]".to_string(),
        Err(e) => return Err(format!("Database error: {}", e)),
    };

    let mut alerts = serde_json::from_str::<Vec<Alert>>(&alerts_json)
        .map_err(|e| format!("Failed to parse alerts JSON: {}", e))?;
    
    // Update the alert in the list
    for a in &mut alerts {
        if a.id == alert.id {
            a.firing = alert.firing;
            break;
        }
    }
    
    // Save back to database
    let updated_json = serde_json::to_string(&alerts)
        .map_err(|e| format!("Failed to serialize alerts: {}", e))?;
    
    db.set_kv_str("alerts", &updated_json)
        .map_err(|e| format!("Failed to save alerts: {}", e))
}

/// Send a notification through the configured method
async fn send_notification(method: &NotificationMethod, message: &str) -> Result<(), String> {
    match &method.config {
        NotificationConfig::WebHook(webhook) => send_webhook_notification(webhook, message).await
    }
}

/// Send a webhook notification
async fn send_webhook_notification(webhook: &WebHookNotif, message: &str) -> Result<(), String> {
    let client = Client::builder().use_rustls_tls().build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    // Replace placeholder with actual message
    let url = webhook.url.replace("{notif_msg}", message);
    
    // Prepare headers
    let mut headers = reqwest::header::HeaderMap::new();
    for (key, value) in &webhook.headers {
        headers.insert(
            reqwest::header::HeaderName::from_bytes(key.as_bytes())
                .map_err(|e| format!("Invalid header name: {}", e))?,
            reqwest::header::HeaderValue::from_str(value)
                .map_err(|e| format!("Invalid header value: {}", e))?,
        );
    }
    
    // Build request based on method
    let mut request_builder = match webhook.method.to_uppercase().as_str() {
        "GET" => client.get(&url),
        "POST" => {
            let body = webhook.body.replace("{notif_msg}", message);
            client.post(&url).body(body)
        },
        "PUT" => {
            let body = webhook.body.replace("{notif_msg}", message);
            client.put(&url).body(body)
        },
        "PATCH" => {
            let body = webhook.body.replace("{notif_msg}", message);
            client.patch(&url).body(body)
        },
        "DELETE" => client.delete(&url),
        _ => return Err(format!("Unsupported HTTP method: {}", webhook.method)),
    };
    
    // Add headers
    request_builder = request_builder.headers(headers);
    
    // Send request
    let response = request_builder.send()
        .await
        .map_err(|e| format!("Failed to send webhook request: {:?}", e))?;
    
    if response.status().is_success() {
        info!("Webhook notification sent successfully");
        Ok(())
    } else {
        Err(format!("Webhook request failed with status: {}", response.status()))
    }
}

/// Get a user-friendly name for a variable
fn get_var_friendly_name(var: &AlertVar) -> String {
    match (var.cat.as_str(), var.var.as_str()) {
        ("sys", "cpu_usage") => "CPU Usage".to_string(),
        ("sys", "mem_usage") => "Memory Usage".to_string(),
        ("sys", "swap_usage") => "Swap Usage".to_string(),
        ("sys", "load_avg_1") => "1 Min Load Average".to_string(),
        ("sys", "load_avg_5") => "5 Min Load Average".to_string(),
        ("sys", "load_avg_15") => "15 Min Load Average".to_string(),
        ("net", "rx_rate") => "Network Receive Rate".to_string(),
        ("net", "tx_rate") => "Network Transmit Rate".to_string(),
        ("disk", "read_rate") => "Disk Read Rate".to_string(),
        ("disk", "write_rate") => "Disk Write Rate".to_string(),
        ("disk", "disk_usage") => "Disk Usage".to_string(),
        _ => format!("{} {}", var.cat, var.var),
    }
}

/// Format an alert message with appropriate units and verbs
fn format_alert_message(alert: &Alert, is_firing: bool) -> String {
    // Get the resource identifier if applicable
    let resource = if alert.var.cat != "sys" {
        format!(" ({})", alert.var.resrc)
    } else {
        "".to_string()
    };

    // Get unit for the metric
    let (value_with_unit, verb) = match (alert.var.cat.as_str(), alert.var.var.as_str(), alert.operator.as_str()) {
        // System metrics
        (_, "cpu_usage", ">") => (format!("{}%", alert.threshold), "exceeded"),
        (_, "cpu_usage", "<") => (format!("{}%", alert.threshold), "dropped below"),
        (_, "mem_usage", ">") => (format!("{}%", alert.threshold), "exceeded"),
        (_, "mem_usage", "<") => (format!("{}%", alert.threshold), "dropped below"),
        (_, "swap_usage", ">") => (format!("{}%", alert.threshold), "exceeded"),
        (_, "swap_usage", "<") => (format!("{}%", alert.threshold), "dropped below"),
        (_, "disk_usage", ">") => (format!("{}%", alert.threshold), "exceeded"),
        (_, "disk_usage", "<") => (format!("{}%", alert.threshold), "dropped below"),
        (_, "load_avg_1", ">") => (format!("{}", alert.threshold), "exceeded"),
        (_, "load_avg_1", "<") => (format!("{}", alert.threshold), "dropped below"),
        (_, "load_avg_5", ">") => (format!("{}", alert.threshold), "exceeded"),
        (_, "load_avg_5", "<") => (format!("{}", alert.threshold), "dropped below"),
        (_, "load_avg_15", ">") => (format!("{}", alert.threshold), "exceeded"),
        (_, "load_avg_15", "<") => (format!("{}", alert.threshold), "dropped below"),
        
        // Network metrics
        ("net", "rx_rate", ">") => (format_bytes_per_sec(alert.threshold), "exceeded"),
        ("net", "rx_rate", "<") => (format_bytes_per_sec(alert.threshold), "dropped below"),
        ("net", "tx_rate", ">") => (format_bytes_per_sec(alert.threshold), "exceeded"),
        ("net", "tx_rate", "<") => (format_bytes_per_sec(alert.threshold), "dropped below"),
        
        // Disk metrics
        ("disk", "read_rate", ">") => (format_bytes_per_sec(alert.threshold), "exceeded"),
        ("disk", "read_rate", "<") => (format_bytes_per_sec(alert.threshold), "dropped below"),
        ("disk", "write_rate", ">") => (format_bytes_per_sec(alert.threshold), "exceeded"),
        ("disk", "write_rate", "<") => (format_bytes_per_sec(alert.threshold), "dropped below"),
        
        // Default case
        (_, _, ">") => (format!("{}", alert.threshold), "exceeded"),
        (_, _, "<") => (format!("{}", alert.threshold), "is below"),
        _ => (format!("{}", alert.threshold), "equals"),
    };

    if is_firing {
        format!(
            "ALERT: {}{} {} {} (sustained for {} min)",
            get_var_friendly_name(&alert.var),
            resource,
            verb,
            value_with_unit,
            alert.time_window
        )
    } else {
        format!(
            "RESOLVED: {}{} no longer {} {} (back to normal)",
            get_var_friendly_name(&alert.var),
            resource,
            verb,
            value_with_unit
        )
    }
}

/// Format bytes per second with appropriate units
fn format_bytes_per_sec(bytes_per_sec: f64) -> String {
    const KIB: f64 = 1024.0;
    
    if bytes_per_sec <= 0.0 {
        return "0.00 B/s".to_string();
    }
    
    let units = ["B/s", "KiB/s", "MiB/s", "GiB/s"];
    let exp = (bytes_per_sec.ln() / KIB.ln()).floor() as i32;
    let exp = exp.clamp(0, (units.len() - 1) as i32);
    
    let value = bytes_per_sec / KIB.powi(exp);
    format!("{:.2} {}", value, units[exp as usize])
}
