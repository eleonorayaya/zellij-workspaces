use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};

static LOGGER: OnceLock<Logger> = OnceLock::new();

pub struct Logger {
    tracing_enabled: AtomicBool,
}

impl Logger {
    fn init() -> Self {
        Logger {
            tracing_enabled: AtomicBool::new(false),
        }
    }

    pub fn get() -> &'static Logger {
        LOGGER.get_or_init(|| Logger::init())
    }

    pub fn start_tracing(&self) {
        use std::fs::File;
        use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

        let log_path = "/host/utena.log";
        let file_result = File::create(log_path);

        match file_result {
            Ok(log_file) => {
                let debug_log = tracing_subscriber::fmt::layer().with_writer(Arc::new(log_file));

                if tracing_subscriber::registry()
                    .with(debug_log)
                    .try_init()
                    .is_ok()
                {
                    self.tracing_enabled.store(true, Ordering::Relaxed);
                    tracing::info!("tracing initialized at {:?}", log_path);
                } else {
                    eprintln!("[ERROR] error initializing tracing subscriber");
                }
            }
            Err(error) => {
                eprintln!("[ERROR] error creating log file: {:?}", error);
            }
        }
    }

    pub fn debug(&self, message: String) {
        if self.tracing_enabled.load(Ordering::Relaxed) {
            tracing::debug!("{}", message);
        } else {
            eprintln!("[DEBUG] {}", message);
        }
    }

    pub fn info(&self, message: String) {
        if self.tracing_enabled.load(Ordering::Relaxed) {
            tracing::info!("{}", message);
        } else {
            eprintln!("[INFO] {}", message);
        }
    }

    pub fn warn(&self, message: String) {
        if self.tracing_enabled.load(Ordering::Relaxed) {
            tracing::warn!("{}", message);
        } else {
            eprintln!("[WARN] {}", message);
        }
    }

    pub fn error(&self, message: String) {
        if self.tracing_enabled.load(Ordering::Relaxed) {
            tracing::error!("{}", message);
        } else {
            eprintln!("[ERROR] {}", message);
        }
    }
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::logger::Logger::get().debug(format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::logger::Logger::get().info(format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::logger::Logger::get().warn(format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::logger::Logger::get().error(format!($($arg)*))
    };
}
