/*
 * Copyright (c) 2025-2026, Adel Noureddine.
 * All rights reserved. This program and the accompanying materials
 * are made available under the terms of the
 * GNU General Public License v3.0 only (GPL-3.0-only)
 * which accompanies this distribution, and is available at
 * https://www.gnu.org/licenses/gpl-3.0.en.html
 *
 * Author : Adel Noureddine
 */

#![windows_subsystem = "windows"]

pub mod gui;

use clap::Parser;
use joularcore::{args::Args, common, logging, monitor::JoularCoreMonitor};


use std::time::Duration;

const CPU_IDLE_CALIBRATION_SAMPLES: usize = 5;
const CPU_IDLE_CALIBRATION_INTERVAL_SECS: u64 = 1;

fn main() {
    logging::init();

    let mut args = Args::parse();

    #[cfg(not(feature = "api"))]
    if args.api_port.is_some() {
        logging::print_error(
            "--api-port is unavailable because this binary was compiled without the API feature",
        );
        std::process::exit(2);
    }

    // Force GUI mode to ensure logic consistent with GUI expectations (though we run run_gui directly below)
    args.gui = true;

    // Initialize platform logic and other components (VM, API, etc.)
    let ctx = common::setup_joularcore(&args);
    let common::JoularContext {
        cpu_energy,
        gpu_energy,
        platform,
        ringbuffer,
        api_sender,
        api_shutdown_tx,
    } = ctx;
    let cpu_usage = platform.cpu_usage();
    let process_util = platform.process_cpu_usage();
    let app_util =
        platform.app_cpu_usage(std::time::Duration::from_secs(args.app_refresh_interval));

    let mut monitor = JoularCoreMonitor::new(
        platform,
        cpu_energy,
        gpu_energy,
        cpu_usage,
        process_util,
        app_util,
        args.cpu_idle_baseline,
    );

    if args.calibrate_cpu_idle_baseline {
        eprintln!(
            "Calibrating CPU idle baseline for the GUI over {} seconds. Keep the machine idle.",
            CPU_IDLE_CALIBRATION_SAMPLES
        );
        let baseline = monitor.calibrate_cpu_idle_baseline(
            CPU_IDLE_CALIBRATION_SAMPLES,
            Duration::from_secs(CPU_IDLE_CALIBRATION_INTERVAL_SECS),
        );
        eprintln!("Calibrated CPU idle baseline: {:.2} W", baseline);
    }

    // Run the GUI directly
    if let Err(e) = gui::run_gui(
        monitor,
        ringbuffer,
        api_sender,
        api_shutdown_tx,
        args.app_refresh_interval,
        args.api_port,
        args.api_allowed_origins.clone(),
    ) {
        logging::print_error(&format!("Failed to launch Joular Core GUI: {}", e));
    }
}
