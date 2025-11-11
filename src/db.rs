use crate::models::{
    ALERT_VARIABLES, AlertVar, GeneralInfo, HistoricalQueryOptions, HistoricalSeries,
};
use log::error;
use rusqlite::{Connection, Result, params};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use sysinfo::System;

use crate::collect_info::collect_general_info;

const STORE_INTERVAL: u64 = 2;
pub struct Database {
    pub conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        // Enable Write-Ahead Logging for better concurrency and performance
        let _ = conn.query_row("PRAGMA journal_mode = WAL", [], |row| {
            row.get::<_, String>(0)
        })?;
        conn.execute("PRAGMA synchronous = NORMAL", [])?;
        conn.execute("PRAGMA cache_size = 1000", [])?;
        conn.execute("PRAGMA temp_store = FILE", [])?;

        // Create tables
        for table_name in ["general_s", "general_m", "general_h", "general_d"] {
            conn.execute(
                format!(
                    "CREATE TABLE IF NOT EXISTS {} (
                    id INTEGER PRIMARY KEY,
                    timestamp INTEGER,
                    cpu_usage REAL,
                    mem_usage REAL,
                    swap_usage REAL,
                    load_avg_1 REAL,
                    load_avg_5 REAL,
                    load_avg_15 REAL
                )",
                    table_name
                )
                .as_str(),
                [],
            )?;
        }

        for table_name in ["net_s", "net_m", "net_h", "net_d"] {
            conn.execute(
                format!(
                    "CREATE TABLE IF NOT EXISTS {} (
                    id INTEGER PRIMARY KEY,
                    timestamp INTEGER,
                    name TEXT,
                    rx REAL,
                    tx REAL,
                    rx_rate REAL,
                    tx_rate REAL
                )",
                    table_name
                )
                .as_str(),
                [],
            )?;
        }

        for table_name in ["disk_s", "disk_m", "disk_h", "disk_d"] {
            conn.execute(
                format!(
                    "CREATE TABLE IF NOT EXISTS {} (
                    id INTEGER PRIMARY KEY,
                    timestamp INTEGER,
                    name TEXT,
                    total_read REAL,
                    total_write REAL,
                    read_rate REAL,
                    write_rate REAL,
                    disk_usage REAL
                )",
                    table_name
                )
                .as_str(),
                [],
            )?;
        }

        conn.execute(
            "CREATE TABLE IF NOT EXISTS kv (
                key TEXT PRIMARY KEY,
                value BLOB
            )",
            [],
        )?;

        // Create indexes
        // Timestamp indexes for all tables
        for table in [
            "general_s",
            "general_m",
            "general_h",
            "general_d",
            "net_s",
            "net_m",
            "net_h",
            "net_d",
            "disk_s",
            "disk_m",
            "disk_h",
            "disk_d",
        ] {
            conn.execute(
                &format!(
                    "CREATE INDEX IF NOT EXISTS idx_{}_timestamp ON {} (timestamp)",
                    table, table
                ),
                [],
            )?;
        }

        // Compound indexes for network and disk tables to optimize queries by name and timestamp
        for table in [
            "net_s", "net_m", "net_h", "net_d", "disk_s", "disk_m", "disk_h", "disk_d",
        ] {
            conn.execute(
                &format!(
                    "CREATE INDEX IF NOT EXISTS idx_{}_name_timestamp ON {} (name, timestamp)",
                    table, table
                ),
                [],
            )?;
        }

        conn.execute("CREATE INDEX IF NOT EXISTS idx_kv_key ON kv (key)", [])?;

        Ok(Database {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn get_kv_str(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT value FROM kv WHERE key = ?")?;
        let mut rows = stmt.query(params![key])?;
        match rows.next()? {
            Some(row) => Ok(Some(row.get(0)?)),
            _ => Ok(None),
        }
    }

    pub fn set_kv_str(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO kv (key, value) VALUES (?, ?)",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn query_historical_data(
        &self,
        options: &HistoricalQueryOptions,
    ) -> Result<Vec<HistoricalSeries>> {
        let resolution = match options.resolution.as_str() {
            "second" => "s",
            "minute" => "m",
            "hour" => "h",
            "day" => "d",
            _ => "m", // Default to minute metrics
        };

        let mut series_results: Vec<HistoricalSeries> = Vec::new();

        for cat in ["general", "net", "disk"] {
            let table_name = format!("{}_{}", cat, resolution);
            // Build the query
            let mut query = format!("SELECT * FROM {}", table_name);
            let mut query_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

            // Fix query construction - first condition shouldn't have AND
            let mut has_where = false;
            if let Some(start) = options.start_time {
                query.push_str(" WHERE timestamp >= ?");
                query_params.push(Box::new(start));
                has_where = true;
            }

            if let Some(end) = options.end_time {
                if has_where {
                    query.push_str(" AND timestamp <= ?");
                } else {
                    query.push_str(" WHERE timestamp <= ?");
                }
                query_params.push(Box::new(end));
            }

            query.push_str(" ORDER BY timestamp");

            if let Some(limit) = options.limit {
                query.push_str(" LIMIT ?");
                query_params.push(Box::new(limit));
            }

            // create HistoricalSeries from query results for each column
            let mut series_map: std::collections::HashMap<(String, String), HistoricalSeries> =
                std::collections::HashMap::new();

            let conn = self.conn.lock().unwrap();
            let mut stmt = match conn.prepare(&query) {
                Ok(stmt) => stmt,
                Err(e) => {
                    error!("Failed to prepare query for {}: {}", table_name, e);
                    continue;
                }
            };

            // Get column names
            let column_names: Vec<String> =
                stmt.column_names().iter().map(|s| s.to_string()).collect();

            // Create parameter references
            let param_refs: Vec<&dyn rusqlite::ToSql> =
                query_params.iter().map(|p| p.as_ref()).collect();

            // Execute the query
            let mut rows = match stmt.query(rusqlite::params_from_iter(param_refs.iter())) {
                Ok(rows) => rows,
                Err(e) => {
                    error!("Failed to execute query for {}: {}", table_name, e);
                    continue;
                }
            };

            // Process rows
            while let Some(row) = rows.next().ok().flatten() {
                let timestamp: i64 = row.get(1)?; // timestamp is always index 1

                if cat == "general" {
                    // General tables have no name column, use "system" as name
                    for (idx, col_name) in column_names.iter().enumerate().skip(2) {
                        // Skip id and timestamp
                        if let Ok(value) = row.get::<_, f64>(idx) {
                            let key = (col_name.clone(), "system".to_string());
                            let entry = series_map.entry(key).or_insert_with(|| HistoricalSeries {
                                cat: cat.to_string(),
                                stype: col_name.clone(),
                                name: "system".to_string(),
                                timestamps: Vec::new(),
                                values: Vec::new(),
                            });

                            entry.timestamps.push(timestamp);
                            entry.values.push(value);
                        }
                    }
                } else {
                    // Net and disk tables have name column at index 2
                    let name: String = match row.get(2) {
                        Ok(name) => name,
                        Err(_) => continue, // Skip if name can't be retrieved
                    };

                    for (idx, col_name) in column_names.iter().enumerate().skip(3) {
                        // skip(3) to skip id, timestamp, name
                        if ["rx", "tx", "total_read", "total_write"].contains(&col_name.as_str()) {
                            // Skip total read/write columns
                            continue;
                        }
                        // Skip id, timestamp, name
                        if let Ok(value) = row.get::<_, f64>(idx) {
                            let key = (col_name.clone(), name.clone());
                            let entry = series_map.entry(key).or_insert_with(|| HistoricalSeries {
                                cat: cat.to_string(),
                                stype: col_name.clone(),
                                name: name.clone(),
                                timestamps: Vec::new(),
                                values: Vec::new(),
                            });

                            entry.timestamps.push(timestamp);
                            entry.values.push(value);
                        }
                    }
                }
            }

            // Add all series from this table to the results
            series_results.extend(series_map.into_values());
        }
        Ok(series_results)
    }

    pub fn get_resource_list(&self) -> Result<Vec<AlertVar>> {
        let mut alert_vars: Vec<AlertVar> = Vec::new();

        // get all names in net_s and disk_s
        for c in ["net", "disk"] {
            let conn = self.conn.lock().unwrap();
            let mut stmt = conn.prepare(&format!("SELECT DISTINCT name FROM {}_s", c))?;
            let rows = stmt.query_map([], |row| row.get(0))?;
            let resources: Vec<String> = rows.collect::<Result<Vec<String>>>()?;

            let cols: Vec<&(&str, &str)> = ALERT_VARIABLES.iter().filter(|v| v.0 == c).collect();

            // every resource_col combination
            for resource in resources.clone() {
                for &(_, col) in cols.iter() {
                    alert_vars.push(AlertVar {
                        cat: c.to_string(),
                        var: col.to_string(),
                        resrc: resource.clone(),
                    });
                }
            }
        }

        // system vars
        let cols: Vec<&(&str, &str)> = ALERT_VARIABLES.iter().filter(|v| v.0 == "sys").collect();
        for &(_, col) in cols.iter() {
            alert_vars.push(AlertVar {
                cat: "sys".to_string(),
                var: col.to_string(),
                resrc: "sys".to_string(),
            });
        }

        Ok(alert_vars)
    }
}

pub async fn db_update(sys: Arc<Mutex<System>>, db_path: &str) {
    let db = match Database::new(db_path) {
        Ok(db) => Arc::new(db),
        Err(e) => {
            error!("Failed to initialize database: {}", e);
            return;
        }
    };
    let mut last_info: Option<GeneralInfo> = None;
    let mut last_timestamp: Option<u64> = None;
    loop {
        let general_info = {
            let sys = sys.lock().unwrap();
            collect_general_info(&sys)
        };
        {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let conn = db.conn.lock().unwrap();
            conn.execute(
                "INSERT INTO general_s (
                    timestamp, cpu_usage, mem_usage, swap_usage, load_avg_1, load_avg_5, load_avg_15
                ) VALUES (?, ROUND(?,2), ROUND(?,2), ROUND(?,2), ROUND(?,2), ROUND(?,2), ROUND(?,2))",
                params![
                    timestamp,
                    general_info.cpu.avg_usage,
                    100.0 * general_info.mem.used_mem as f32 / general_info.mem.total_mem as f32,
                    100.0 * general_info.mem.used_swap as f32 / general_info.mem.total_swap as f32,
                    general_info.sys.load_avg[0],
                    general_info.sys.load_avg[1],
                    general_info.sys.load_avg[2]
                ],
            )
            .unwrap();

            for interface in general_info.net.interfaces.iter() {
                let mut rx_rate = 0.0;
                let mut tx_rate = 0.0;
                // check if last info is initialized
                if let (Some(last_info), Some(last_timestamp)) = (&last_info, last_timestamp) {
                    // Find the matching interface in last_info
                    let last_interface = last_info
                        .net
                        .interfaces
                        .iter()
                        .find(|last_iface| last_iface.name == interface.name);

                    if let Some(last_iface) = last_interface {
                        // Calculate time difference in seconds
                        let elapsed_secs = timestamp as f64 - last_timestamp as f64;

                        // Calculate rates
                        rx_rate = if interface.rx > last_iface.rx {
                            (interface.rx - last_iface.rx) as f64 / elapsed_secs
                        } else {
                            0.0
                        };
                        tx_rate = if interface.tx > last_iface.tx {
                            (interface.tx - last_iface.tx) as f64 / elapsed_secs
                        } else {
                            0.0
                        };
                    }
                }
                conn.execute(
                    "INSERT INTO net_s (
                        timestamp, name, rx, tx, rx_rate, tx_rate
                    ) VALUES (?, ?, ?, ?, ROUND(?), ROUND(?))",
                    params![
                        timestamp,
                        interface.name,
                        interface.rx,
                        interface.tx,
                        rx_rate,
                        tx_rate
                    ],
                )
                .unwrap();
            }

            for disk in general_info.disk.disks.iter() {
                let mut read_rate = 0.0;
                let mut write_rate = 0.0;
                // check if last info is initialized
                if let (Some(last_info), Some(last_timestamp)) = (&last_info, last_timestamp) {
                    // Find the matching disk in last_info
                    let last_disk = last_info
                        .disk
                        .disks
                        .iter()
                        .find(|last_disk| last_disk.mount_point == disk.mount_point);

                    if let Some(last_disk) = last_disk {
                        // Calculate time difference in seconds
                        let elapsed_secs = timestamp as f64 - last_timestamp as f64;

                        // Calculate rates
                        read_rate = if disk.io[2] > last_disk.io[2] {
                            (disk.io[2] - last_disk.io[2]) as f64 / elapsed_secs
                        } else {
                            0.0
                        };
                        write_rate = if disk.io[3] > last_disk.io[3] {
                            (disk.io[3] - last_disk.io[3]) as f64 / elapsed_secs
                        } else {
                            0.0
                        };
                    }
                }
                conn.execute(
                    "INSERT INTO disk_s (
                        timestamp, name, total_read, total_write, read_rate, write_rate, disk_usage
                    ) VALUES (?, ?, ?, ?, ROUND(?), ROUND(?), ROUND(?,2))",
                    params![
                        timestamp,
                        disk.mount_point,
                        disk.io[2],
                        disk.io[3],
                        read_rate,
                        write_rate,
                        100.0 * (1.0 - disk.free_space as f32 / disk.total_space as f32)
                    ],
                )
                .unwrap();
            }

            // if skipped over the minute mark still need to aggregate the last minute's data

            if timestamp % 60 < STORE_INTERVAL {
                // We have passed (or are on) a minute boundary
                // Set timestamp to the minute boundary
                let timestamp = timestamp - (timestamp % 60);
                // Aggregate last minute's data
                let _ = conn.execute(
                    "INSERT INTO general_m
                                    (
                                    timestamp,
                                    cpu_usage,
                                    mem_usage,
                                    swap_usage,
                                    load_avg_1,
                                    load_avg_5,
                                    load_avg_15
                                    )
                                    SELECT 
                                        ?2,
                                        round(AVG(cpu_usage), 2),
                                        round(AVG(mem_usage), 2),
                                        round(AVG(swap_usage), 2),
                                        round(AVG(load_avg_1), 2),
                                        round(AVG(load_avg_5), 2),
                                        round(AVG(load_avg_15), 2)
                                    FROM general_s
                                    WHERE timestamp >= ?1 AND timestamp <= ?2;",
                    params![timestamp - 60, timestamp],
                );
                let _ = conn.execute(
                    "INSERT INTO net_m
                                    (
                                    timestamp,
                                    name,
                                    rx,
                                    tx,
                                    rx_rate,
                                    tx_rate
                                    )
                                    SELECT 
                                        ?2,
                                        name,
                                        MAX(rx),
                                        MAX(tx),
                                        round(AVG(rx_rate)),
                                        round(AVG(tx_rate))
                                    FROM net_s
                                    WHERE timestamp >= ?1 AND timestamp <= ?2
                                    GROUP BY name;",
                    params![timestamp - 60, timestamp],
                );
                let _ = conn.execute(
                    "INSERT INTO disk_m
                                    (
                                    timestamp,
                                    name,
                                    total_read,
                                    total_write,
                                    read_rate,
                                    write_rate,
                                    disk_usage
                                    )
                                    SELECT 
                                        ?2,
                                        name,
                                        MAX(total_read),
                                        MAX(total_write),
                                        round(AVG(read_rate)),
                                        round(AVG(write_rate)),
                                        round(AVG(disk_usage), 2)
                                    FROM disk_s
                                    WHERE timestamp >= ?1 AND timestamp <= ?2
                                    GROUP BY name;",
                    params![timestamp - 60, timestamp],
                );

                // Check if it's an hour boundary
                if (timestamp / 60).is_multiple_of(60) {
                    // Aggregate minute_metrics for the last hour
                    let _ = conn.execute(
                        "INSERT INTO general_h
                                        (
                                        timestamp,
                                        cpu_usage,
                                        mem_usage,
                                        swap_usage,
                                        load_avg_1,
                                        load_avg_5,
                                        load_avg_15
                                        )
                                        SELECT 
                                            ?2,
                                            round(AVG(cpu_usage)),
                                            round(AVG(mem_usage)),
                                            round(AVG(swap_usage)),
                                            round(AVG(load_avg_1)),
                                            round(AVG(load_avg_5)),
                                            round(AVG(load_avg_15))
                                        FROM general_m
                                        WHERE timestamp >= ?1 AND timestamp <= ?2;",
                        params![timestamp - 3600, timestamp],
                    );
                    let _ = conn.execute(
                        "INSERT INTO net_h
                                        (
                                        timestamp,
                                        name,
                                        rx,
                                        tx,
                                        rx_rate,
                                        tx_rate
                                        )
                                        SELECT 
                                            ?2,
                                            name,
                                            MAX(rx),
                                            MAX(tx),
                                            round(AVG(rx_rate)),
                                            round(AVG(tx_rate))
                                        FROM net_m
                                        WHERE timestamp >= ?1 AND timestamp <= ?2
                                        GROUP BY name;",
                        params![timestamp - 3600, timestamp],
                    );
                    let _ = conn.execute(
                        "INSERT INTO disk_h
                                        (
                                        timestamp,
                                        name,
                                        total_read,
                                        total_write,
                                        read_rate,
                                        write_rate,
                                        disk_usage
                                        )
                                        SELECT 
                                            ?2,
                                            name,
                                            MAX(total_read),
                                            MAX(total_write),
                                            round(AVG(read_rate)),
                                            round(AVG(write_rate)),
                                            round(AVG(disk_usage), 2)
                                        FROM disk_m
                                        WHERE timestamp >= ?1 AND timestamp <= ?2
                                        GROUP BY name;",
                        params![timestamp - 3600, timestamp],
                    );
                    // Check if it's a day boundary (midnight)
                    if (timestamp / 3600).is_multiple_of(24) {
                        // Aggregate hour_metrics for the last day
                        let _ = conn.execute(
                            "INSERT INTO general_d
                                            (
                                            timestamp,
                                            cpu_usage,
                                            mem_usage,
                                            swap_usage,
                                            load_avg_1,
                                            load_avg_5,
                                            load_avg_15
                                            )
                                            SELECT 
                                                ?2,
                                                round(AVG(cpu_usage)),
                                                round(AVG(mem_usage)),
                                                round(AVG(swap_usage)),
                                                round(AVG(load_avg_1)),
                                                round(AVG(load_avg_5)),
                                                round(AVG(load_avg_15))
                                            FROM general_h
                                            WHERE timestamp >= ?1 AND timestamp <= ?2;",
                            params![timestamp - 86400, timestamp],
                        );
                        let _ = conn.execute(
                            "INSERT INTO net_d
                                            (
                                            timestamp,
                                            name,
                                            rx,
                                            tx,
                                            rx_rate,
                                            tx_rate
                                            )
                                            SELECT 
                                                ?2,
                                                name,
                                                MAX(rx),
                                                MAX(tx),
                                                round(AVG(rx_rate)),
                                                round(AVG(tx_rate))
                                            FROM net_h
                                            WHERE timestamp >= ?1 AND timestamp <= ?2
                                            GROUP BY name;",
                            params![timestamp - 86400, timestamp],
                        );
                        let _ = conn.execute(
                            "INSERT INTO disk_d
                                            (
                                            timestamp,
                                            name,
                                            total_read,
                                            total_write,
                                            read_rate,
                                            write_rate,
                                            disk_usage
                                            )
                                            SELECT 
                                                ?2,
                                                name,
                                                MAX(total_read),
                                                MAX(total_write),
                                                round(AVG(read_rate)),
                                                round(AVG(write_rate)),
                                                round(AVG(disk_usage), 2)
                                            FROM disk_h
                                            WHERE timestamp >= ?1 AND timestamp <= ?2
                                            GROUP BY name;",
                            params![timestamp - 86400, timestamp],
                        );

                        // Clean up older hour metrics (keep 365 days)
                        let cutoff = timestamp - (86400 * 365);
                        for table_name in ["general_h", "net_h", "disk_h"] {
                            conn.execute(
                                format!("DELETE FROM {} WHERE timestamp < ?", table_name).as_str(),
                                params![cutoff],
                            )
                            .unwrap();
                        }
                        // Run VACCUM
                        conn.execute("VACUUM", []).unwrap();
                        conn.execute("pragma optimize", []).unwrap();
                    }
                    // Clean up older second data (keep 1 hours)
                    let cutoff = timestamp - 3600;
                    for table_name in ["general_s", "net_s", "disk_s"] {
                        conn.execute(
                            format!("DELETE FROM {} WHERE timestamp < ?", table_name).as_str(),
                            params![cutoff],
                        )
                        .unwrap();
                    }
                    // Clean up older minute metrics (keep 96 hours)
                    let cutoff = timestamp - (86400 * 4);
                    for table_name in ["general_m", "net_m", "disk_m"] {
                        conn.execute(
                            format!("DELETE FROM {} WHERE timestamp < ?", table_name).as_str(),
                            params![cutoff],
                        )
                        .unwrap();
                    }
                }
            }
            last_info = Some(general_info.clone());
            last_timestamp = Some(timestamp);
        }

        tokio::time::sleep(Duration::from_secs(STORE_INTERVAL)).await;
    }
}
