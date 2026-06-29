pub mod action;
pub mod app;
mod bytes;
pub mod cli;
pub mod components;
pub mod config;
mod diff;
mod exec;
pub mod mode;
mod old_config;
mod runner;
mod search;
mod store;
mod termtext;
pub mod tui;
mod types;
mod utils;
mod widget;

use clap::Parser;
use cli::Cli;
use color_eyre::eyre::{Result, eyre};

use crate::{
    app::App,
    utils::{initialize_logging, install_eyre_hook},
};

async fn tokio_main() -> Result<()> {
    install_eyre_hook()?;

    let args = Cli::parse();

    if args.load.is_none() && args.command.is_empty() {
        return Err(eyre!("No command provided"));
    }
    if args.load.is_some() && args.command.len() > 1 {
        return Err(eyre!("Can not use --load with command"));
    }

    if args.disable_auto_save {
        let store = store::memory::MemoryStore::new();
        let mut app = App::new(args, store, false)?;
        app.run().await?;
    } else if let Some(l) = &args.load {
        let store = store::sqlite::SQLiteStore::new(l.clone(), false)?;
        let mut app = App::new(args.clone(), store, true)?;
        app.run().await?;
    } else if let Some(b) = &args.save {
        let store = store::sqlite::SQLiteStore::new(b.clone(), true)?;
        let mut app = App::new(args.clone(), store, false)?;
        app.run().await?;
    } else {
        // Create the temporary directory with owner-only permissions so the
        // backup (which may contain sensitive output) is not world-readable (#195).
        #[cfg(unix)]
        let tmp_dir = {
            use std::os::unix::fs::PermissionsExt;
            tempfile::Builder::new()
                .permissions(std::fs::Permissions::from_mode(0o700))
                .tempdir()?
        };
        #[cfg(not(unix))]
        let tmp_dir = tempfile::tempdir()?;
        let tmp_path = tmp_dir.keep();
        let file_path = tmp_path.join("backup.sqlite");
        let store = store::sqlite::SQLiteStore::new(file_path.clone(), true)?;
        let mut app = App::new(args.clone(), store, false)?;
        app.run().await?;

        println!("Backup saved at {}", file_path.to_str().unwrap());
        println!(
            "Run `viddy --lookback {}` to load backup",
            file_path.to_str().unwrap()
        );
    }

    Ok(())
}

fn main() -> Result<()> {
    unsafe {
        // Safety: The caller must ensure this is called in a single-threaded program.
        initialize_logging()
            .inspect_err(|_| eprintln!("{} error: Something went wrong", env!("CARGO_PKG_NAME")))?;
    }
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            tokio_main().await.inspect_err(|_| {
                eprintln!("{} error: Something went wrong", env!("CARGO_PKG_NAME"));
            })
        })
}
