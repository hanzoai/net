use chrono::Local;

use std::sync::{Arc, Mutex, Once};

// Conditional compilation: Only include tracing imports for non-WASM targets
#[cfg(not(target_arch = "wasm32"))]
use tracing::{debug, error, info, span, Level};

static INIT: Once = Once::new();
static TELEMETRY: Mutex<Option<Arc<dyn ZooTelemetry + Send + Sync>>> = Mutex::new(None);

pub fn set_telemetry(telemetry: Arc<dyn ZooTelemetry + Send + Sync>) {
    let mut telemetry_option = TELEMETRY.lock().unwrap();
    *telemetry_option = Some(telemetry);
}

pub trait ZooTelemetry {
    fn log(&self, option: ZooLogOption, level: ZooLogLevel, message: &str);
}

#[derive(PartialEq, Debug)]
pub enum ZooLogOption {
    Blockchain,
    Database,
    Identity,
    IdentityNetwork,
    ExtSubscriptions,
    MySubscriptions,
    SubscriptionHTTPUploader,
    SubscriptionHTTPDownloader,
    CryptoIdentity,
    JobExecution,
    CronExecution,
    Api,
    WsAPI,
    DetailedAPI,
    Node,
    InternalAPI,
    Network,
    Tests,
}

#[derive(PartialEq)]
pub enum ZooLogLevel {
    Error,
    Info,
    Debug,
}

impl ZooLogLevel {
    // Conditional compilation: Only include function for non-WASM targets
    #[cfg(not(target_arch = "wasm32"))]
    #[allow(dead_code)]
    fn to_log_level(&self) -> Level {
        match self {
            ZooLogLevel::Error => Level::ERROR,
            ZooLogLevel::Info => Level::INFO,
            ZooLogLevel::Debug => Level::DEBUG,
        }
    }
}

fn active_log_options() -> Vec<ZooLogOption> {
    if std::env::var("LOG_ALL").is_ok() {
        return vec![
            ZooLogOption::Blockchain,
            ZooLogOption::Database,
            ZooLogOption::Identity,
            ZooLogOption::IdentityNetwork,
            ZooLogOption::ExtSubscriptions,
            ZooLogOption::MySubscriptions,
            ZooLogOption::SubscriptionHTTPUploader,
            ZooLogOption::SubscriptionHTTPDownloader,
            ZooLogOption::CryptoIdentity,
            ZooLogOption::JobExecution,
            ZooLogOption::CronExecution,
            ZooLogOption::Api,
            ZooLogOption::WsAPI,
            ZooLogOption::DetailedAPI,
            ZooLogOption::Node,
            ZooLogOption::InternalAPI,
            ZooLogOption::Network,
            ZooLogOption::Tests,
        ];
    }

    let mut active_options = Vec::new();
    if std::env::var("LOG_BLOCKCHAIN").is_ok() {
        active_options.push(ZooLogOption::Blockchain);
    }
    if std::env::var("LOG_DATABASE").is_ok() {
        active_options.push(ZooLogOption::Database);
    }
    if std::env::var("LOG_IDENTITY").is_ok() {
        active_options.push(ZooLogOption::Identity);
    }
    if std::env::var("LOG_IDENTITY_NETWORK").is_ok() {
        active_options.push(ZooLogOption::IdentityNetwork);
    }
    if std::env::var("LOG_EXT_SUBSCRIPTIONS").is_ok() {
        active_options.push(ZooLogOption::ExtSubscriptions);
    }
    if std::env::var("LOG_MY_SUBSCRIPTIONS").is_ok() {
        active_options.push(ZooLogOption::MySubscriptions);
    }
    if std::env::var("LOG_SUBSCRIPTION_HTTP_UPLOADER").is_ok() {
        active_options.push(ZooLogOption::SubscriptionHTTPUploader);
    }
    if std::env::var("LOG_SUBSCRIPTION_HTTP_DOWNLOADER").is_ok() {
        active_options.push(ZooLogOption::SubscriptionHTTPDownloader);
    }
    if std::env::var("LOG_CRYPTO_IDENTITY").is_ok() {
        active_options.push(ZooLogOption::CryptoIdentity);
    }
    if std::env::var("LOG_API").is_ok() {
        active_options.push(ZooLogOption::Api);
    }
    if std::env::var("LOG_WS_API").is_ok() {
        active_options.push(ZooLogOption::WsAPI);
    }
    if std::env::var("LOG_DETAILED_API").is_ok() {
        active_options.push(ZooLogOption::DetailedAPI);
    }
    if std::env::var("LOG_NODE").is_ok() {
        active_options.push(ZooLogOption::Node);
    }
    if std::env::var("LOG_INTERNAL_API").is_ok() {
        active_options.push(ZooLogOption::InternalAPI);
    }
    if std::env::var("LOG_INTERNAL_NETWORK").is_ok() {
        active_options.push(ZooLogOption::Network);
    }
    if std::env::var("LOG_TESTS").is_ok() {
        active_options.push(ZooLogOption::Tests);
    }
    if std::env::var("LOG_JOB_EXECUTION").is_ok() {
        active_options.push(ZooLogOption::JobExecution);
    }
    if std::env::var("LOG_CRON_EXECUTION").is_ok() {
        active_options.push(ZooLogOption::CronExecution);
    }
    active_options
}

pub fn zoo_log(option: ZooLogOption, level: ZooLogLevel, message: &str) {
    let active_options = active_log_options();
    if active_options.contains(&option) {
        let is_simple_log = std::env::var("LOG_SIMPLE").is_ok();
        let time = Local::now().format("%Y-%m-%d %H:%M:%S");

        let option_str = format!("{:?}", option);
        let level_str = match level {
            ZooLogLevel::Error => "ERROR",
            ZooLogLevel::Info => "INFO",
            ZooLogLevel::Debug => "DEBUG",
        };

        let message_with_header = if is_simple_log {
            message.to_string()
        } else {
            let hostname = "localhost";
            let app_name = "zoo";
            let proc_id = std::process::id().to_string();
            let msg_id = "-";
            let header = format!("{} {} {} {} {}", time, hostname, app_name, proc_id, msg_id);
            format!("{} - {} - {} - {}", header, level_str, option_str, message)
        };

        // Conditional compilation: Only include tracing-related code for non-WASM targets
        #[cfg(not(target_arch = "wasm32"))]
        {
            let span = match level {
                ZooLogLevel::Error => span!(Level::ERROR, "{}", option_str),
                ZooLogLevel::Info => span!(Level::INFO, "{}", option_str),
                ZooLogLevel::Debug => span!(Level::DEBUG, "{}", option_str),
            };

            span.in_scope(|| {
                let telemetry_option = TELEMETRY.lock().unwrap();
                match telemetry_option.as_ref() {
                    Some(telemetry) => {
                        telemetry.log(option, level, &message_with_header);
                    }
                    None => match level {
                        ZooLogLevel::Error => error!("{}", message_with_header),
                        ZooLogLevel::Info => info!("{}", message_with_header),
                        ZooLogLevel::Debug => debug!("{}", message_with_header),
                    },
                }
            });
        }
    }
}

pub fn init_default_tracing() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        INIT.call_once(|| {
            let log_var = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

            // Determine the most permissive log level specified
            let filter_level = if log_var.contains("trace") {
                "trace"
            } else if log_var.contains("debug") {
                "debug"
            } else if log_var.contains("info") {
                "info"
            } else if log_var.contains("warn") {
                "warn"
            } else if log_var.contains("error") {
                "error"
            } else {
                "info" // Default to info if none specified or recognized
            };

            let filter = tracing_subscriber::EnvFilter::new(filter_level);

            let subscriber = tracing_subscriber::fmt::Subscriber::builder()
                .with_env_filter(filter)
                .with_timer(tracing_subscriber::fmt::time::time())
                .with_target(true) // disables printing of the target
                .finish();

            let _ = tracing::subscriber::set_global_default(subscriber);
        });
    }
}
