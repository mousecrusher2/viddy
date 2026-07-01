use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use color_eyre::eyre::Result;
use directories::{BaseDirs, ProjectDirs};
use tracing_error::ErrorLayer;
use tracing_subscriber::{
    self, Layer,
    filter::{EnvFilter, LevelFilter},
    prelude::__tracing_subscriber_SubscriberExt,
    util::SubscriberInitExt,
};

const DATA_ENV: &str = concat!(env!("PACKAGE_ENV_PREFIX"), "_DATA");
const CONFIG_ENV: &str = concat!(env!("PACKAGE_ENV_PREFIX"), "_CONFIG");
const LOG_ENV: &str = concat!(env!("PACKAGE_ENV_PREFIX"), "_LOGLEVEL");

const LOG_FILE: &str = concat!(env!("CARGO_PKG_NAME"), ".log");
static PROJECT_DIRECTORY: LazyLock<Option<ProjectDirs>> =
    LazyLock::new(|| ProjectDirs::from("dev", "sachaos", env!("CARGO_PKG_NAME")));
static DATA_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    if let Ok(path) = std::env::var(DATA_ENV) {
        PathBuf::from(path)
    } else if let Some(ref proj_dirs) = *PROJECT_DIRECTORY {
        proj_dirs.data_local_dir().to_path_buf()
    } else {
        PathBuf::from(".").join(".data")
    }
});
static CONFIG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    if let Ok(path) = std::env::var(CONFIG_ENV) {
        PathBuf::from(path)
    } else if let Some(ref proj_dirs) = *PROJECT_DIRECTORY {
        proj_dirs.config_local_dir().to_path_buf()
    } else {
        PathBuf::from(".").join(".config")
    }
});
static OLD_CONFIG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    if let Some(base_dirs) = BaseDirs::new() {
        base_dirs.config_dir().to_path_buf()
    } else {
        PathBuf::from(".").join(".config")
    }
});

pub fn install_eyre_hook() -> Result<()> {
    let (_, eyre_hook) = color_eyre::config::HookBuilder::default()
        .capture_span_trace_by_default(false)
        .display_location_section(false)
        .display_env_section(false)
        .into_hooks();
    eyre_hook.install()?;
    Ok(())
}

#[must_use]
pub fn get_data_dir() -> &'static Path {
    DATA_DIR.as_path()
}

#[must_use]
pub fn get_config_dir() -> &'static Path {
    CONFIG_DIR.as_path()
}

fn logging_filter(env_var: impl Fn(&str) -> Option<String>) -> EnvFilter {
    let rust_log = env_var(EnvFilter::DEFAULT_ENV);
    let package_log_level = env_var(LOG_ENV);
    let default_filter = format!("{}=info", env!("CARGO_CRATE_NAME"));
    let log_filter = rust_log
        .as_deref()
        .or(package_log_level.as_deref())
        .unwrap_or(default_filter.as_str());

    EnvFilter::builder()
        .with_default_directive(LevelFilter::ERROR.into())
        .parse_lossy(log_filter)
}

pub fn initialize_logging() -> Result<()> {
    let directory = get_data_dir();
    std::fs::create_dir_all(directory)?;
    let log_path = directory.join(LOG_FILE);
    let log_file = std::fs::File::create(log_path)?;
    let file_subscriber = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_writer(log_file)
        .with_target(false)
        .with_ansi(false)
        .with_filter(logging_filter(|key| std::env::var(key).ok()));
    tracing_subscriber::registry()
        .with(file_subscriber)
        .with(ErrorLayer::default())
        .init();
    Ok(())
}

#[must_use]
pub fn get_old_config_dir() -> &'static Path {
    OLD_CONFIG_DIR.as_path()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[derive(Clone, Copy)]
    struct EnvCase {
        value: Option<&'static str>,
        selected_filter: &'static str,
    }

    const ENV_CASES: &[EnvCase] = &[
        EnvCase {
            value: None,
            selected_filter: concat!(env!("CARGO_CRATE_NAME"), "=info"),
        },
        EnvCase {
            value: Some(""),
            selected_filter: "error",
        },
        EnvCase {
            value: Some(concat!(env!("CARGO_CRATE_NAME"), "=bogus")),
            selected_filter: "error",
        },
        EnvCase {
            value: Some("off"),
            selected_filter: "off",
        },
        EnvCase {
            value: Some("error"),
            selected_filter: "error",
        },
        EnvCase {
            value: Some("warn"),
            selected_filter: "warn",
        },
        EnvCase {
            value: Some("info"),
            selected_filter: "info",
        },
        EnvCase {
            value: Some("debug"),
            selected_filter: "debug",
        },
        EnvCase {
            value: Some("trace"),
            selected_filter: "trace",
        },
        EnvCase {
            value: Some(concat!(env!("CARGO_CRATE_NAME"), "=off")),
            selected_filter: concat!(env!("CARGO_CRATE_NAME"), "=off"),
        },
        EnvCase {
            value: Some(concat!(env!("CARGO_CRATE_NAME"), "=error")),
            selected_filter: concat!(env!("CARGO_CRATE_NAME"), "=error"),
        },
        EnvCase {
            value: Some(concat!(env!("CARGO_CRATE_NAME"), "=warn")),
            selected_filter: concat!(env!("CARGO_CRATE_NAME"), "=warn"),
        },
        EnvCase {
            value: Some(concat!(env!("CARGO_CRATE_NAME"), "=info")),
            selected_filter: concat!(env!("CARGO_CRATE_NAME"), "=info"),
        },
        EnvCase {
            value: Some(concat!(env!("CARGO_CRATE_NAME"), "=debug")),
            selected_filter: concat!(env!("CARGO_CRATE_NAME"), "=debug"),
        },
        EnvCase {
            value: Some(concat!(env!("CARGO_CRATE_NAME"), "=trace")),
            selected_filter: concat!(env!("CARGO_CRATE_NAME"), "=trace"),
        },
        EnvCase {
            value: Some("other_crate=off"),
            selected_filter: "other_crate=off",
        },
        EnvCase {
            value: Some("other_crate=error"),
            selected_filter: "other_crate=error",
        },
        EnvCase {
            value: Some("other_crate=warn"),
            selected_filter: "other_crate=warn",
        },
        EnvCase {
            value: Some("other_crate=info"),
            selected_filter: "other_crate=info",
        },
        EnvCase {
            value: Some("other_crate=debug"),
            selected_filter: "other_crate=debug",
        },
        EnvCase {
            value: Some("other_crate=trace"),
            selected_filter: "other_crate=trace",
        },
        EnvCase {
            value: Some(concat!(env!("CARGO_CRATE_NAME"), "=bogus,debug")),
            selected_filter: "debug",
        },
        EnvCase {
            value: Some(concat!(
                env!("CARGO_CRATE_NAME"),
                "=bogus,",
                env!("CARGO_CRATE_NAME"),
                "=debug"
            )),
            selected_filter: concat!(env!("CARGO_CRATE_NAME"), "=debug"),
        },
        EnvCase {
            value: Some(concat!(
                env!("CARGO_CRATE_NAME"),
                "=bogus,other_crate=debug"
            )),
            selected_filter: "other_crate=debug",
        },
    ];

    #[test]
    fn logging_filter_preserves_rust_log_fallback_behavior_for_all_env_states() {
        for rust_log in ENV_CASES {
            for package_log_level in ENV_CASES {
                let read_rust_log = Cell::new(false);
                let read_package_log_level = Cell::new(false);

                let filter = logging_filter(|key| match key {
                    EnvFilter::DEFAULT_ENV => {
                        read_rust_log.set(true);
                        rust_log.value.map(ToString::to_string)
                    }
                    key if key == LOG_ENV => {
                        read_package_log_level.set(true);
                        package_log_level.value.map(ToString::to_string)
                    }
                    _ => unreachable!("unexpected env var key: {key}"),
                });
                let expected_filter = rust_log
                    .value
                    .map_or(package_log_level.selected_filter, |_| {
                        rust_log.selected_filter
                    });

                assert_eq!(
                    filter.to_string(),
                    expected_filter,
                    "RUST_LOG={:?}, {LOG_ENV}={:?}",
                    rust_log.value,
                    package_log_level.value,
                );
                assert!(
                    read_rust_log.get(),
                    "RUST_LOG was not read for RUST_LOG={:?}, {LOG_ENV}={:?}",
                    rust_log.value,
                    package_log_level.value,
                );
                assert!(
                    read_package_log_level.get(),
                    "{LOG_ENV} was not read for RUST_LOG={:?}, {LOG_ENV}={:?}",
                    rust_log.value,
                    package_log_level.value,
                );
            }
        }
    }
}
