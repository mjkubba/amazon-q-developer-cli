pub mod cli;
pub mod util;

use std::process::ExitCode;

use anstream::eprintln;
use clap::Parser;
use clap::error::{
    ContextKind,
    ErrorKind,
};
#[cfg(not(feature = "minimal"))]
use crossterm::style::Stylize;
use eyre::Result;
use fig_log::get_log_level_max;
use fig_util::{
    CLI_BINARY_NAME,
    PRODUCT_NAME,
};
use tracing::metadata::LevelFilter;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(not(feature = "mimalloc"))]
#[global_allocator]
static GLOBAL: std::alloc::System = std::alloc::System;

fn main() -> Result<ExitCode> {
    color_eyre::install()?;
    fig_telemetry::set_dispatch_mode(fig_telemetry::DispatchMode::On);
    fig_telemetry::init_global_telemetry_emitter();

    let multithread = matches!(
        std::env::var("Q_SINGLE_THREADED"),
        Err(_) | Ok(ref s) if s.is_empty() || s == "0" || s == "false"
    );

    let rt = if multithread {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?
    } else {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?
    };

    rt.block_on(async {
        let result = cli::run().await;
        fig_telemetry::finish_telemetry().await;
        result
    })
}
