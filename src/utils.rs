use std::{path::PathBuf, sync::LazyLock};

use color_eyre::eyre::Result;
use directories::{BaseDirs, ProjectDirs};
use tracing_error::ErrorLayer;
use tracing_subscriber::{
    self, Layer, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt,
};

const VERSION_MESSAGE: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "-",
    env!("VERGEN_GIT_DESCRIBE"),
    " (",
    env!("VERGEN_BUILD_DATE"),
    ")"
);

pub static PROJECT_NAME: LazyLock<String> =
    LazyLock::new(|| env!("CARGO_CRATE_NAME").to_uppercase());
pub static DATA_FOLDER: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
    std::env::var(format!("{}_DATA", *PROJECT_NAME))
        .ok()
        .map(PathBuf::from)
});
pub static CONFIG_FOLDER: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
    std::env::var(format!("{}_CONFIG", *PROJECT_NAME))
        .ok()
        .map(PathBuf::from)
});
pub static LOG_ENV: LazyLock<String> = LazyLock::new(|| format!("{}_LOGLEVEL", *PROJECT_NAME));
pub const LOG_FILE: &str = concat!(env!("CARGO_PKG_NAME"), ".log");

pub static PROJECT_DIRECTORY: LazyLock<Option<ProjectDirs>> =
    LazyLock::new(|| ProjectDirs::from("dev", "sachaos", env!("CARGO_PKG_NAME")));

pub fn initialize_panic_handler() -> Result<()> {
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

#[must_use]
pub fn get_old_config_dir() -> PathBuf {
    if let Some(base_dirs) = BaseDirs::new() {
        base_dirs.config_dir().to_path_buf()
    } else {
        PathBuf::from(".").join(".config")
    }
}

/// # Safety
/// This function is safe to call in a single-threaded program.
pub unsafe fn initialize_logging() -> Result<()> {
    let directory = get_data_dir();
    std::fs::create_dir_all(directory.clone())?;
    let log_path = directory.join(LOG_FILE);
    let log_file = std::fs::File::create(log_path)?;
    unsafe {
        // Safety: The caller must ensure this is called in a single-threaded program.
        std::env::set_var(
            "RUST_LOG",
            std::env::var("RUST_LOG")
                .or_else(|_| std::env::var(LOG_ENV.clone()))
                .unwrap_or_else(|_| format!("{}=info", env!("CARGO_CRATE_NAME"))),
        );
    };
    let file_subscriber = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_writer(log_file)
        .with_target(false)
        .with_ansi(false)
        .with_filter(tracing_subscriber::filter::EnvFilter::from_default_env());
    tracing_subscriber::registry()
        .with(file_subscriber)
        .with(ErrorLayer::default())
        .init();
    Ok(())
}

/// Similar to the `std::dbg!` macro, but generates `tracing` events rather
/// than printing to stdout.
///
/// By default, the verbosity level for the generated events is `DEBUG`, but
/// this can be customized.
#[macro_export]
macro_rules! trace_dbg {
    (target: $target:expr, level: $level:expr, $ex:expr) => {{
        match $ex {
            value => {
                tracing::event!(target: $target, $level, ?value, stringify!($ex));
                value
            }
        }
    }};
    (level: $level:expr, $ex:expr) => {
        trace_dbg!(target: module_path!(), level: $level, $ex)
    };
    (target: $target:expr, $ex:expr) => {
        trace_dbg!(target: $target, level: tracing::Level::DEBUG, $ex)
    };
    ($ex:expr) => {
        trace_dbg!(level: tracing::Level::DEBUG, $ex)
    };
}

#[must_use]
pub fn version() -> String {
    let author = clap::crate_authors!();

    // let current_exe_path = PathBuf::from(clap::crate_name!()).display().to_string();
    let config_dir_path = get_config_dir().display().to_string();
    let data_dir_path = get_data_dir().display().to_string();

    format!(
        "\
{VERSION_MESSAGE}

Authors: {author}

Config directory: {config_dir_path}
Data directory: {data_dir_path}"
    )
}
