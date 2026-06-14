use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager, State};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

pub struct VpnProcessState(pub Mutex<Option<(String, Child)>>);

#[derive(Debug, Serialize)]
pub struct ProfileLogSnapshot {
    pub content: String,
    pub modified_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ProfileConfig {
    pub token: String,
    pub protocol: String,
    pub fakeUrls: String,
    pub methods: String,
    pub endPoints: String,
    pub timeout: String,
    pub pullTimeout: String,
    pub tunnelEnable: bool,
    pub connectionReuse: bool,
    pub tlsEnable: bool,
    pub tlsVerifyPeer: bool,
    pub tlsCertFile: String,
    pub tlsKeyFile: String,
    pub tlsCaFile: String,
    pub logLevel: String,
    pub listenIp: String,
    pub listenPort: String,
    pub serverIp: String,
    pub serverPort: String,
    pub httpVersion: String,
    pub userAgent: String,
    pub bufferSize: String,
}

fn profile_config_to_yaml(config: &ProfileConfig) -> String {
    let fake_urls = config
        .fakeUrls
        .lines()
        .map(|s| format!("    - {}", s))
        .collect::<Vec<_>>()
        .join("\n");

    let methods = config
        .methods
        .lines()
        .map(|s| format!("    - {}", s))
        .collect::<Vec<_>>()
        .join("\n");

    let endpoints = config
        .endPoints
        .lines()
        .map(|s| format!("    - {}", s))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"---
general:
  token: "{}"
  protocol: {}
  bufferSize: {}
  fakeUrls:
{}
  methods:
{}
  endPoints:
{}
  timeout: {}
  pullTimeout: {}
  tunnelEnable: {}
  connectionReuse: {}
  tlsEnable: {}
  tlsVerifyPeer: {}
  tlsCertFile: "{}"
  tlsKeyFile: "{}"
  tlsCaFile: "{}"

log:
  logLevel: "{}"
  logFile: "nipovpn.log"

server:
  threads: 8
  listenIp: "0.0.0.0"
  listenPort: 80

agent:
  threads: 8
  listenIp: "{}"
  listenPort: {}
  serverIp: "{}"
  serverPort: {}
  httpVersion: "{}"
  userAgent: "{}"
"#,
        config.token,
        config.protocol,
        config.bufferSize,
        fake_urls,
        methods,
        endpoints,
        config.timeout,
        config.pullTimeout,
        config.tunnelEnable,
        config.connectionReuse,
        config.tlsEnable,
        config.tlsVerifyPeer,
        config.tlsCertFile,
        config.tlsKeyFile,
        config.tlsCaFile,
        config.logLevel,
        config.listenIp,
        config.listenPort,
        config.serverIp,
        config.serverPort,
        config.httpVersion,
        config.userAgent,
    )
}

const PING_TARGET_HOST: &str = "google.com";
const PING_TARGET_PORT: u16 = 443;
const PROXY_HOST: &str = "127.0.0.1";
const TIMEOUT_MS: u64 = 5000;

fn connect_proxy(proxy_port: u16) -> io::Result<TcpStream> {
    let addr = format!("{}:{}", PROXY_HOST, proxy_port)
        .parse()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Invalid proxy address"))?;

    TcpStream::connect_timeout(&addr, Duration::from_millis(TIMEOUT_MS))
}

fn read_status_line(stream: &mut TcpStream) -> io::Result<String> {
    stream.set_read_timeout(Some(Duration::from_millis(TIMEOUT_MS)))?;

    let mut reader = BufReader::new(stream);
    let mut status_line = String::new();
    reader.read_line(&mut status_line)?;

    Ok(status_line.trim_end_matches(['\r', '\n']).to_string())
}

fn measure_http_ping(proxy_port: u16) -> io::Result<u64> {
    let started_at = Instant::now();
    let mut stream = connect_proxy(proxy_port)?;

    stream.set_write_timeout(Some(Duration::from_millis(TIMEOUT_MS)))?;
    stream.set_read_timeout(Some(Duration::from_millis(TIMEOUT_MS)))?;

    let request = format!(
        "CONNECT {}:{} HTTP/1.1\r\nHost: {}:{}\r\nProxy-Connection: close\r\nUser-Agent: NipoVPN-Desktop\r\n\r\n",
        PING_TARGET_HOST, PING_TARGET_PORT, PING_TARGET_HOST, PING_TARGET_PORT
    );

    stream.write_all(request.as_bytes())?;
    stream.flush()?;

    let status_line = read_status_line(&mut stream)?;
    let is_success = status_line.contains(" 200 ")
        || status_line.starts_with("HTTP/1.1 200")
        || status_line.starts_with("HTTP/1.0 200");

    if is_success {
        Ok(started_at.elapsed().as_millis() as u64)
    } else {
        Err(io::Error::new(
            io::ErrorKind::ConnectionRefused,
            format!("HTTP CONNECT failed with status: {}", status_line),
        ))
    }
}

fn read_exact_or_timeout(stream: &mut TcpStream, buffer: &mut [u8]) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_millis(TIMEOUT_MS)))?;
    stream.read_exact(buffer)
}

fn measure_socks5_ping(proxy_port: u16) -> io::Result<u64> {
    let started_at = Instant::now();
    let mut stream = connect_proxy(proxy_port)?;

    stream.set_write_timeout(Some(Duration::from_millis(TIMEOUT_MS)))?;

    stream.write_all(&[0x05, 0x01, 0x00])?;
    stream.flush()?;

    let mut handshake = [0u8; 2];
    read_exact_or_timeout(&mut stream, &mut handshake)?;

    if handshake[0] != 0x05 || handshake[1] != 0x00 {
        return Err(io::Error::new(
            io::ErrorKind::ConnectionRefused,
            "SOCKS5 handshake rejected by proxy",
        ));
    }

    let domain = PING_TARGET_HOST.as_bytes();
    if domain.len() > u8::MAX as usize {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Target host is too long for SOCKS5 domain request",
        ));
    }

    let mut connect_request = Vec::with_capacity(7 + domain.len());
    connect_request.push(0x05);
    connect_request.push(0x01);
    connect_request.push(0x00);
    connect_request.push(0x03);
    connect_request.push(domain.len() as u8);
    connect_request.extend_from_slice(domain);
    connect_request.extend_from_slice(&PING_TARGET_PORT.to_be_bytes());

    stream.write_all(&connect_request)?;
    stream.flush()?;

    let mut reply_header = [0u8; 4];
    read_exact_or_timeout(&mut stream, &mut reply_header)?;

    if reply_header[0] != 0x05 || reply_header[1] != 0x00 {
        return Err(io::Error::new(
            io::ErrorKind::ConnectionRefused,
            "SOCKS5 connect request rejected by proxy",
        ));
    }

    match reply_header[3] {
        0x01 => {
            let mut ipv4 = [0u8; 4];
            read_exact_or_timeout(&mut stream, &mut ipv4)?;
        }
        0x03 => {
            let mut domain_len = [0u8; 1];
            read_exact_or_timeout(&mut stream, &mut domain_len)?;
            let mut bound_domain = vec![0u8; domain_len[0] as usize];
            read_exact_or_timeout(&mut stream, &mut bound_domain)?;
        }
        0x04 => {
            let mut ipv6 = [0u8; 16];
            read_exact_or_timeout(&mut stream, &mut ipv6)?;
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid SOCKS5 bound address type",
            ));
        }
    }

    let mut _bound_port = [0u8; 2];
    read_exact_or_timeout(&mut stream, &mut _bound_port)?;

    Ok(started_at.elapsed().as_millis() as u64)
}

#[tauri::command]
pub fn measure_ping(proxy_port: u16, protocol: String) -> Result<u64, String> {
    log::debug!(target: "commands", "measure_ping called: proxy_port={}, protocol={}", proxy_port, protocol);
    let result = match protocol.to_ascii_lowercase().as_str() {
        "http" => measure_http_ping(proxy_port).map_err(|e| e.to_string()),
        "socks5" => measure_socks5_ping(proxy_port).map_err(|e| e.to_string()),
        _ => Err("Unsupported protocol".into()),
    };
    match &result {
        Ok(latency_ms) => {
            log::info!(target: "commands", "measure_ping succeeded: {}ms", latency_ms)
        }
        Err(err) => log::warn!(target: "commands", "measure_ping failed: {}", err),
    }
    result
}

fn resolve_profile_log_path(app: &AppHandle, profile_id: &str) -> Result<PathBuf, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(app_data_dir
        .join("profiles")
        .join(profile_id)
        .join("nipovpn.log"))
}

fn file_modified_ms(path: &Path) -> Option<i64> {
    std::fs::metadata(path)
        .ok()?
        .modified()
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis().min(i64::MAX as u128) as i64)
}

fn read_profile_log_snapshot(
    app: &AppHandle,
    profile_id: &str,
) -> Result<ProfileLogSnapshot, String> {
    let log_path = resolve_profile_log_path(app, profile_id)?;

    if !log_path.exists() {
        return Ok(ProfileLogSnapshot {
            content: String::new(),
            modified_ms: None,
        });
    }

    let content = std::fs::read_to_string(&log_path)
        .map_err(|e| format!("Failed to read log file: {}", e))?;

    Ok(ProfileLogSnapshot {
        content,
        modified_ms: file_modified_ms(&log_path),
    })
}

fn resolve_system_log_path() -> Result<PathBuf, String> {
    let exe_dir = crate::logger::resolve_exe_dir().map_err(|e| e.to_string())?;
    Ok(crate::logger::log_file_path(&exe_dir))
}

fn read_system_log_snapshot() -> Result<ProfileLogSnapshot, String> {
    let log_path = resolve_system_log_path()?;

    if !log_path.exists() {
        return Ok(ProfileLogSnapshot {
            content: String::new(),
            modified_ms: None,
        });
    }

    let content = std::fs::read_to_string(&log_path)
        .map_err(|e| format!("Failed to read system log file: {}", e))?;

    Ok(ProfileLogSnapshot {
        content,
        modified_ms: file_modified_ms(&log_path),
    })
}

#[tauri::command]
pub fn start_vpn(
    app: AppHandle,
    profile_id: String,
    config: ProfileConfig,
) -> Result<String, String> {
    let resource_path = app
        .path()
        .resource_dir()
        .map_err(|e| e.to_string())?
        .join("bin/nipovpn/nipovpn.exe");

    if !resource_path.exists() {
        return Err("NipoVPN executable not found in resources".into());
    }

    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let profile_dir = app_data_dir.join("profiles").join(&profile_id);
    std::fs::create_dir_all(&profile_dir).map_err(|e| e.to_string())?;

    let log_path = profile_dir.join("nipovpn.log");
    std::fs::write(&log_path, b"").map_err(|e| format!("Failed to clear log file: {}", e))?;

    let yaml_content = profile_config_to_yaml(&config);
    let config_path = profile_dir.join("config.yaml");
    std::fs::write(&config_path, &yaml_content).map_err(|e| e.to_string())?;

    let mut cmd = Command::new(&resource_path);
    cmd.current_dir(&profile_dir)
        .arg("agent")
        .arg(&config_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(target_os = "windows")]
    {
        cmd.creation_flags(0x08000000);
    }

    log::info!(target: "commands", "Starting VPN for profile '{}' with config: {}", profile_id, config_path.display());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            log::error!(target: "commands", "Failed to start VPN for profile '{}': {}", profile_id, e);
            return Err(format!("Failed to start agent: {}", e));
        }
    };

    if let Some(stdout) = child.stdout.take() {
        let app_clone = app.clone();
        std::thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(text) = line {
                    let _ = app_clone.emit("nipovpn-log", &text);
                }
            }
        });
    }

    if let Some(stderr) = child.stderr.take() {
        let app_clone = app.clone();
        std::thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                if let Ok(text) = line {
                    let _ = app_clone.emit("nipovpn-log", &text);
                }
            }
        });
    }

    let state: State<VpnProcessState> = app.state();
    let mut lock = state.0.lock().unwrap();
    *lock = Some((profile_id.clone(), child));

    Ok(format!(
        "VPN started for profile '{}' with config at: {}",
        profile_id,
        config_path.display()
    ))
}

#[tauri::command]
pub fn stop_vpn(state: State<VpnProcessState>) -> Result<String, String> {
    let mut lock = state.0.lock().unwrap();
    if let Some((profile_id, mut child)) = lock.take() {
        log::info!(target: "commands", "Stopping VPN for profile '{}'", profile_id);
        let _ = child.kill();
        Ok(format!("VPN stopped for profile '{}'", profile_id))
    } else {
        log::debug!(target: "commands", "stop_vpn called but no VPN process is currently active");
        Err("No VPN process is currently running".into())
    }
}

#[tauri::command]
pub fn get_profile_log(app: AppHandle, profile_id: String) -> Result<ProfileLogSnapshot, String> {
    read_profile_log_snapshot(&app, &profile_id)
}

#[tauri::command]
pub fn get_system_log() -> Result<ProfileLogSnapshot, String> {
    read_system_log_snapshot()
}

#[tauri::command]
pub fn clear_profile_log(app: AppHandle, profile_id: String) -> Result<String, String> {
    let log_path = resolve_profile_log_path(&app, &profile_id)?;

    std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&log_path)
        .map_err(|e| format!("Failed to clear log file: {}", e))?;

    Ok(String::new())
}

#[tauri::command]
pub fn clear_system_log() -> Result<String, String> {
    let log_path = resolve_system_log_path()?;

    std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&log_path)
        .map_err(|e| format!("Failed to clear system log file: {}", e))?;

    Ok(String::new())
}

#[tauri::command]
pub fn get_running_profile(state: State<VpnProcessState>) -> Result<Option<String>, String> {
    let lock = state.0.lock().unwrap();
    Ok(lock.as_ref().map(|(id, _)| id.clone()))
}

#[tauri::command]
pub fn delete_profile_config(app: AppHandle, profile_id: String) -> Result<String, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let profile_dir = app_data_dir.join("profiles").join(&profile_id);

    if profile_dir.exists() {
        std::fs::remove_dir_all(&profile_dir).map_err(|e| e.to_string())?;
        Ok(format!("Profile '{}' configuration deleted", profile_id))
    } else {
        Ok(format!("Profile '{}' configuration not found", profile_id))
    }
}

pub(crate) fn parse_log_level(level: &str) -> Option<log::Level> {
    match level.to_ascii_lowercase().as_str() {
        "trace" => Some(log::Level::Trace),
        "debug" => Some(log::Level::Debug),
        "info" => Some(log::Level::Info),
        "warn" => Some(log::Level::Warn),
        "error" => Some(log::Level::Error),
        _ => None,
    }
}

#[tauri::command]
pub fn log_message(level: String, module: String, message: String) {
    let module = if module.trim().is_empty() {
        "unknown".to_string()
    } else {
        module
    };

    let tagged_message = format!("[FRONTEND/{}] {}", module, message);

    match parse_log_level(&level) {
        Some(log_level) => {
            log::log!(target: "frontend", log_level, "{}", tagged_message);
        }
        None => {
            let prefixed = format!("[original-level: {}] {}", level, tagged_message);
            log::info!(target: "frontend", "{}", prefixed);
        }
    }
}
