use std::{path::PathBuf, sync::LazyLock};

use color_eyre::eyre::Result;
use directories::{BaseDirs, ProjectDirs};
use tracing_error::ErrorLayer;
use tracing_subscriber::{
    self, Layer,
    filter::{EnvFilter, LevelFilter},
    prelude::__tracing_subscriber_SubscriberExt,
    util::SubscriberInitExt,
};

static PROJECT_NAME: LazyLock<String> = LazyLock::new(|| env!("CARGO_CRATE_NAME").to_uppercase());
static DATA_FOLDER: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
    std::env::var(format!("{}_DATA", *PROJECT_NAME))
        .ok()
        .map(PathBuf::from)
});
static CONFIG_FOLDER: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
    std::env::var(format!("{}_CONFIG", *PROJECT_NAME))
        .ok()
        .map(PathBuf::from)
});
static LOG_ENV: LazyLock<String> = LazyLock::new(|| format!("{}_LOGLEVEL", *PROJECT_NAME));
const LOG_FILE: &str = concat!(env!("CARGO_PKG_NAME"), ".log");
static PROJECT_DIRECTORY: LazyLock<Option<ProjectDirs>> =
    LazyLock::new(|| ProjectDirs::from("dev", "sachaos", env!("CARGO_PKG_NAME")));

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
pub fn get_data_dir() -> PathBuf {
    if let Some(s) = &*DATA_FOLDER {
        s.clone()
    } else if let Some(ref proj_dirs) = *PROJECT_DIRECTORY {
        proj_dirs.data_local_dir().to_path_buf()
    } else {
        PathBuf::from(".").join(".data")
    }
}

#[must_use]
pub fn get_config_dir() -> PathBuf {
    if let Some(s) = &*CONFIG_FOLDER {
        s.clone()
    } else if let Some(ref proj_dirs) = *PROJECT_DIRECTORY {
        proj_dirs.config_local_dir().to_path_buf()
    } else {
        PathBuf::from(".").join(".config")
    }
}

fn logging_filter(env_var: impl Fn(&str) -> Option<String>) -> EnvFilter {
    let rust_log = env_var(EnvFilter::DEFAULT_ENV);
    let viddy_log_level = env_var(LOG_ENV.as_str());
    let default_filter = format!("{}=info", env!("CARGO_CRATE_NAME"));
    let log_filter = rust_log
        .as_deref()
        .or(viddy_log_level.as_deref())
        .unwrap_or(default_filter.as_str());

    EnvFilter::builder()
        .with_default_directive(LevelFilter::ERROR.into())
        .parse_lossy(log_filter)
}

pub fn initialize_logging() -> Result<()> {
    let directory = get_data_dir();
    std::fs::create_dir_all(directory.clone())?;
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
pub fn get_old_config_dir() -> PathBuf {
    if let Some(base_dirs) = BaseDirs::new() {
        base_dirs.config_dir().to_path_buf()
    } else {
        PathBuf::from(".").join(".config")
    }
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
            selected_filter: "viddy=info",
        },
        EnvCase {
            value: Some(""),
            selected_filter: "error",
        },
        EnvCase {
            value: Some("viddy=bogus"),
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
            value: Some("viddy=off"),
            selected_filter: "viddy=off",
        },
        EnvCase {
            value: Some("viddy=error"),
            selected_filter: "viddy=error",
        },
        EnvCase {
            value: Some("viddy=warn"),
            selected_filter: "viddy=warn",
        },
        EnvCase {
            value: Some("viddy=info"),
            selected_filter: "viddy=info",
        },
        EnvCase {
            value: Some("viddy=debug"),
            selected_filter: "viddy=debug",
        },
        EnvCase {
            value: Some("viddy=trace"),
            selected_filter: "viddy=trace",
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
            value: Some("viddy=bogus,debug"),
            selected_filter: "debug",
        },
        EnvCase {
            value: Some("viddy=bogus,viddy=debug"),
            selected_filter: "viddy=debug",
        },
        EnvCase {
            value: Some("viddy=bogus,other_crate=debug"),
            selected_filter: "other_crate=debug",
        },
    ];

    #[test]
    fn logging_filter_preserves_rust_log_fallback_behavior_for_all_env_states() {
        for rust_log in ENV_CASES {
            for viddy_log_level in ENV_CASES {
                let read_rust_log = Cell::new(false);
                let read_viddy_log_level = Cell::new(false);

                let filter = logging_filter(|key| match key {
                    EnvFilter::DEFAULT_ENV => {
                        read_rust_log.set(true);
                        rust_log.value.map(ToString::to_string)
                    }
                    key if key == LOG_ENV.as_str() => {
                        read_viddy_log_level.set(true);
                        viddy_log_level.value.map(ToString::to_string)
                    }
                    _ => unreachable!("unexpected env var key: {key}"),
                });
                let expected_filter =
                    rust_log.value.map_or(viddy_log_level.selected_filter, |_| {
                        rust_log.selected_filter
                    });

                assert_eq!(
                    filter.to_string(),
                    expected_filter,
                    "RUST_LOG={:?}, VIDDY_LOGLEVEL={:?}",
                    rust_log.value,
                    viddy_log_level.value,
                );
                assert!(
                    read_rust_log.get(),
                    "RUST_LOG was not read for RUST_LOG={:?}, VIDDY_LOGLEVEL={:?}",
                    rust_log.value,
                    viddy_log_level.value,
                );
                assert!(
                    read_viddy_log_level.get(),
                    "VIDDY_LOGLEVEL was not read for RUST_LOG={:?}, VIDDY_LOGLEVEL={:?}",
                    rust_log.value,
                    viddy_log_level.value,
                );
            }
        }
    }
}
