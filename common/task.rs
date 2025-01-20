use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};
use sysinfo::{
    Disks, Pid, ProcessRefreshKind, ProcessesToUpdate, System as sys, System,
};

fn default_i64_0() -> i64 {
    0
}

fn default_u64_0() -> u64 {
    0
}

fn default_none_u64() -> Option<u64> {
    None
}

fn default_none_string() -> Option<String> {
    None
}

fn default_false() -> bool {
    false
}

fn default_vec_string() -> Vec<String> {
    Vec::new()
}

fn default_map_string_string() -> HashMap<String, String> {
    HashMap::new()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub year: Option<i32>,
    pub month: Option<u32>,
    pub day: Option<u32>,
    pub hour: Option<u32>,
    pub minute: Option<u32>,
    pub second: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncTask {
    #[serde(default = "default_none_u64")]
    pub max_restart: Option<u64>,
    #[serde(default = "default_u64_0")]
    pub has_restart: u64,
    #[serde(default = "default_u64_0")]
    pub started_at: u64,
    #[serde(default = "default_u64_0")]
    pub stopped_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodicTask {
    #[serde(default = "default_u64_0")]
    pub started_after: u64,
    pub interval: u64,
    #[serde(default = "default_u64_0")]
    pub last_run: u64,
    #[serde(default = "default_false")]
    pub sync: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    Scheduled(ScheduledTask),
    Async(AsyncTask),
    Periodic(PeriodicTask),
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Task id (unique)
    pub id: i64,

    /// Task name (unique)
    pub name: String,

    /// Task command
    pub command: String,

    /// Task arguments
    #[serde(default = "default_vec_string")]
    pub args: Vec<String>,

    /// Task group
    pub group: Option<String>,

    /// Task working directory
    pub dir: Option<String>,

    /// Task environment variables
    #[serde(default = "default_map_string_string")]
    pub env: HashMap<String, String>,

    pub stdin: Option<bool>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,

    #[serde(default = "default_created_at")]
    pub created_at: u64,
    pub task_type: TaskType,

    pub pid: Option<u32>,

    #[serde(default = "default_status")]
    pub status: Option<String>,
    pub code: Option<i32>,
}

fn default_created_at() -> u64 {
    let now = SystemTime::now();
    let timestamp = now
        .duration_since(UNIX_EPOCH)
        .expect("Failed to get timestamp")
        .as_secs();
    timestamp
}

fn default_status() -> Option<String> {
    Some("added".to_owned())
}

impl Default for Task {
    fn default() -> Self {
        let now = SystemTime::now();
        let timestamp = now
            .duration_since(UNIX_EPOCH)
            .expect("Failed to get timestamp")
            .as_secs();
        Task {
            id: 0,
            name: "Default".to_string(),
            command: String::new(),
            args: vec![],
            group: None,
            dir: None,
            env: HashMap::new(),
            stdin: None,
            stdout: None,
            stderr: None,
            created_at: timestamp,
            task_type: TaskType::None,
            pid: None,
            status: None,
            code: None,
        }
    }
}

unsafe impl Send for Task {}
unsafe impl Sync for Task {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFlag {
    #[serde(default = "default_i64_0")]
    pub id: i64,
    #[serde(default = "default_none_string")]
    pub name: Option<String>,
    #[serde(default = "default_none_string")]
    pub group: Option<String>,
    #[serde(default = "default_false")]
    pub mat: bool,
}

impl Default for TaskFlag {
    fn default() -> Self {
        TaskFlag {
            id: 0,
            name: Some(String::new()),
            group: None,
            mat: false,
        }
    }
}

unsafe impl Send for TaskFlag {}
unsafe impl Sync for TaskFlag {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tasks {
    pub task: Vec<Task>,
}

// 监控指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Matrix {
    pub system_matrix: SystemMatrix,
    pub task_matrix: TaskMatrix,
}

impl Matrix {
    pub fn new(pid: Pid) -> Self {
        let mut system = sys::new_all();
        let disks = Disks::new_with_refreshed_list();
        system.refresh_all();
        Matrix {
            system_matrix: SystemMatrix::new(&system, disks),
            task_matrix: TaskMatrix::new(pid, &mut system),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemMatrix {
    pub cpu_cnt: u32,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub memory_total: u64,
    pub disk_avail_rate: f64,
    pub os_info: String,
}

impl SystemMatrix {
    pub fn new(system: &System, disks: Disks) -> Self {
        SystemMatrix {
            cpu_cnt: system.cpus().len() as u32,
            cpu_usage: system.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>()
                / system.cpus().len() as f32,
            memory_usage: (system.used_memory() as f32 / system.total_memory() as f32) * 100f32,
            memory_total: system.total_memory() / 1024 / 1024,
            disk_avail_rate: disks.iter().map(|disk| disk.available_space()).sum::<u64>() as f64
                / disks.iter().map(|disk| disk.total_space()).sum::<u64>() as f64
                * 100.0,
            os_info: format!(
                "System-name:{} kernel-version:{} os-version:{} host-name:{}",
                sys::name().unwrap(),
                sys::kernel_version().unwrap(),
                sys::os_version().unwrap(),
                sys::host_name().unwrap()
            ),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskMatrix {
    pub cpu_usage: f32,
    pub memory_usage: u64,
}

impl TaskMatrix {
    pub fn new(pid: Pid, system: &mut System) -> Self {
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing().with_cpu(),
        );
        match system.process(pid) {
            None => Self::default(),
            Some(process) => Self {
                cpu_usage: process.cpu_usage(),
                memory_usage: process.memory() / 1024 / 1024,
            },
        }
    }
}
