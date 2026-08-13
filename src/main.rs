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

pub mod args;
pub mod gui;
pub mod session;

use args::Args;
use clap::Parser;

use std::time::Duration;

const CPU_IDLE_CALIBRATION_SAMPLES: usize = 5;
const CPU_IDLE_CALIBRATION_INTERVAL_SECS: u64 = 1;

fn main() {
    let args = Args::parse();

    // The library prints nothing on its own; it emits `log` records. Without a
    // logger installed, "RAPL is not readable" and friends are discarded, and a
    // sensor that reports nothing does so without explanation.
    //
    // `windows_subsystem = "windows"` means this binary has no console on
    // Windows, so these records only reach a terminal on Linux and macOS. The
    // Monitor screen's warning banner is what tells a Windows user that a
    // sensor is unreadable.
    env_logger::Builder::new()
        .filter_level(args.log_level())
        .parse_default_env()
        .init();

    let mut config = args.config();
    let mut monitor = session::build_monitor(&config);

    if args.calibrate_cpu_idle_baseline {
        eprintln!(
            "Calibrating CPU idle baseline for the GUI over {CPU_IDLE_CALIBRATION_SAMPLES} seconds. Keep the machine idle."
        );

        match monitor.calibrate_cpu_idle_baseline(
            CPU_IDLE_CALIBRATION_SAMPLES,
            Duration::from_secs(CPU_IDLE_CALIBRATION_INTERVAL_SECS),
        ) {
            Ok(baseline) => {
                eprintln!("Calibrated CPU idle baseline: {baseline:.2} W");
                // Calibration set the baseline on this monitor, but the GUI
                // builds a new one whenever a build-time setting changes.
                // Recording it in the config is what carries it across.
                config.cpu_idle_baseline = Some(baseline);
            }
            // A failed calibration leaves the previous baseline in place, so
            // the GUI is still usable — say so and carry on rather than exit.
            Err(e) => eprintln!("Failed to calibrate CPU idle baseline: {e}"),
        }
    }

    // Run the GUI directly
    if let Err(e) = gui::run_gui(monitor, config, args) {
        eprintln!("Failed to launch Joular Core GUI: {e}");
    }
}
