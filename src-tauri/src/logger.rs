use log::LevelFilter;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

static FILE_DEGRADED: AtomicBool = AtomicBool::new(false);

pub fn resolve_exe_dir() -> Result<PathBuf, String> {
    let exe = std::env::current_exe().map_err(|e| format!("current_exe() failed: {}", e))?;
    exe.parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| "executable path has no parent directory".to_string())
}

pub fn log_file_path(exe_dir: &Path) -> PathBuf {
    exe_dir.join("app_system.log")
}

pub(crate) fn make_formatter() -> fern::Dispatch {
    fern::Dispatch::new().format(|out, message, record| {
        let timestamp = chrono::Local::now().format("%Y-%m-%d_%H:%M:%S").to_string();
        let level = record.level().to_string().to_uppercase();
        let msg_str = message.to_string();

        let (mode_module, clean_msg) = if msg_str.starts_with("[FRONTEND/") {
            if let Some(end) = msg_str.find("] ") {
                let inner = &msg_str[1..end];
                let rest = &msg_str[end + 2..];
                (inner.to_string(), rest.to_string())
            } else {
                ("FRONTEND/unknown".to_string(), msg_str.clone())
            }
        } else {
            let target = record.target();
            (format!("BACKEND/{}", target), msg_str)
        };

        out.finish(format_args!(
            "{} [{mode_module}] [{level}] {clean_msg}",
            timestamp
        ))
    })
}

struct DegradedFileWriter {
    file: std::fs::File,
}

impl Write for DegradedFileWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if FILE_DEGRADED.load(Ordering::Relaxed) {
            return Ok(buf.len());
        }
        match self.file.write(buf) {
            Ok(n) => Ok(n),
            Err(err) => {
                if FILE_DEGRADED
                    .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok()
                {
                    eprintln!("WARN: App_Logger file write failed: {}", err);
                }
                Ok(buf.len())
            }
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if FILE_DEGRADED.load(Ordering::Relaxed) {
            return Ok(());
        }
        self.file.flush()
    }
}

pub(crate) fn make_file_sink(file: std::fs::File) -> Box<dyn Write + Send + 'static> {
    Box::new(DegradedFileWriter { file })
}

pub enum InitResult {
    Ok { log_file_path: PathBuf },
    FallbackToStdout { reason: String },
}

pub fn init_app_logger() -> InitResult {
    let level_filter = LevelFilter::Warn;

    let exe_dir = match resolve_exe_dir() {
        Ok(dir) => dir,
        Err(reason) => {
            make_formatter()
                .level(level_filter)
                .chain(std::io::stdout())
                .apply()
                .unwrap_or_else(|err| panic!("Failed to initialize logger: {}", err));
            return InitResult::FallbackToStdout { reason };
        }
    };

    let path = log_file_path(&exe_dir);

    let file = match OpenOptions::new().create(true).append(true).open(&path) {
        Ok(f) => f,
        Err(err) => {
            let reason = format!("Failed to open log file '{}': {}", path.display(), err);
            make_formatter()
                .level(level_filter)
                .chain(std::io::stdout())
                .apply()
                .unwrap_or_else(|e| panic!("Failed to initialize logger: {}", e));
            return InitResult::FallbackToStdout { reason };
        }
    };

    make_formatter()
        .level(level_filter)
        .chain(std::io::stdout())
        .chain(make_file_sink(file))
        .apply()
        .unwrap_or_else(|err| panic!("Failed to initialize logger: {}", err));

    InitResult::Ok {
        log_file_path: path,
    }
}
