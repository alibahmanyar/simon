use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemCapabilities {
    pub cpu: bool,
    pub memory: bool,
    pub swap: bool,
    pub load_average: bool,
    pub network: bool,
    pub disk: bool,
    pub processes: bool,
    pub docker: bool,
    pub file_serving: bool,
}

pub const ALERT_VARIABLES: [(&str, &str); 11] = [
    ("sys", "cpu_usage"),
    ("sys", "mem_usage"),
    ("sys", "swap_usage"),
    ("sys", "load_avg_1"),
    ("sys", "load_avg_5"),
    ("sys", "load_avg_15"),
    ("net", "rx_rate"),
    ("net", "tx_rate"),
    ("disk", "read_rate"),
    ("disk", "write_rate"),
    ("disk", "disk_usage"),
];

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        ApiResponse {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(error: String) -> Self {
        ApiResponse {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}

#[derive(Clone, Serialize)]
pub struct GeneralInfo {
    pub t: i64,
    pub sys: SystemInfo,
    pub mem: MemoryInfo,
    pub cpu: CpuInfo,
    pub net: NetworkInfo,
    pub disk: DisksInfo,
}

#[derive(Clone, Serialize)]
pub struct SystemInfo {
    pub name: String,
    pub kernel_ver: String,
    pub os_ver: String,
    pub os_name: String,
    pub host_name: String,
    pub load_avg: Vec<f64>,
    pub uptime: u64,
}

#[derive(Clone, Serialize)]
pub struct MemoryInfo {
    pub total_mem: u64,
    pub used_mem: u64,
    pub total_swap: u64,
    pub used_swap: u64,
}

#[derive(Clone, Serialize)]
pub struct CpuInfo {
    pub count: usize,
    pub avg_usage: f32,
    pub usage: Vec<f32>,
}

#[derive(Clone, Serialize)]
pub struct NetworkInfo {
    pub interfaces: Vec<NetworkInterface>,
}

#[derive(Clone, Serialize)]
pub struct NetworkInterface {
    pub name: String,
    pub rx: u64,
    pub tx: u64,
}

#[derive(Clone, Serialize)]
pub struct DisksInfo {
    pub disks: Vec<DiskInfo>,
}

#[derive(Clone, Serialize)]
pub struct DiskInfo {
    pub fs: String,
    pub kind: String,
    pub total_space: u64,
    pub free_space: u64,
    pub mount_point: String,
    pub removable: bool,
    pub io: [u64; 4], // [read_bytes, write_bytes, total_read_bytes, total_write_bytes]
}

#[derive(Clone, Serialize)]
pub struct ProcessesInfo {
    pub t: i64,
    pub processes: Vec<ProcessInfo>,
}

#[derive(Clone, Serialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub runtime: u64,
    pub name: String,
    pub mem: u64,
    pub cpu: f32,
    pub stat: String,
    pub cmd: String,
    pub env: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DockerContainer {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub state: String,
    pub created: i64,
    pub ports: Vec<DockerPort>,
    pub cpu_usage: f64,
    pub mem_usage: u64,
    pub mem_limit: u64,
    pub net_io: [u64; 2],
    pub disk_io: [u64; 2],
}

#[derive(Debug, Clone, Serialize)]
pub struct DockerPort {
    pub ip: Option<String>,
    pub priv_port: u16,
    pub pub_port: Option<u16>,
    pub protocol: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DockerInfo {
    pub t: i64,
    pub containers: Vec<DockerContainer>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoricalSeries {
    pub cat: String,
    pub stype: String,
    pub name: String,
    pub timestamps: Vec<i64>,
    pub values: Vec<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoricalQueryOptions {
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub limit: Option<i64>,
    pub resolution: String, // "second", "minute", "hour", "day"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub var: AlertVar,
    pub threshold: f64,
    pub operator: String,
    pub time_window: i64,
    pub enabled: bool,
    pub firing: bool,
    pub notif_methods: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AlertVar {
    pub cat: String,   // Category
    pub var: String,   // variable name (ex. rx_rate)
    pub resrc: String, // Resource name (ex. eth0)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NotificationMethod {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub enabled: bool,
    pub config: NotificationConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum NotificationConfig {
    WebHook(WebHookNotif),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebHookNotif {
    pub url: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct EmailNotif {
    pub server: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from: String,
    pub to: String,
    pub subject: String,
    pub body: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: u64,
    pub permissions: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DirectoryListing {
    pub path: String,
    pub entries: Vec<FileEntry>,
}
