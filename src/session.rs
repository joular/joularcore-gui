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

//! Building a monitor, in the one place both startup and the GUI can reach.
//!
//! `app_match`, `app_refresh_interval` and `elevation` are only read while the
//! platform and its trackers are being constructed, so changing any of them in
//! the Options screen means building a new monitor rather than mutating the
//! running one. That happens away from `main`, which is why this lives here:
//! a rebuild must wire up the same VM sensors that startup did, and forgetting
//! to would silently drop a virtual machine back onto its host's sensors.

use joularcore::MonitorConfig;
use joularcore::monitor::JoularCoreMonitor;

/// Build a monitor for `config`, with VM sensors applied when the environment
/// names them.
///
/// The returned monitor is already primed by the builder, so its first
/// [`JoularCoreMonitor::poll`] carries a real interval.
pub fn build_monitor(config: &MonitorConfig) -> JoularCoreMonitor {
    let builder = JoularCoreMonitor::builder(config);
    #[cfg(feature = "vm")]
    let builder = apply_vm_sensors(builder);
    builder.build()
}

/// Read power from the files a hypervisor writes, when the environment names
/// them. The library supplies the sensor; wiring it in is the program's choice,
/// so an unreadable VM file falls back to the platform's own sensor rather than
/// leaving the session with no reading at all.
#[cfg(feature = "vm")]
fn apply_vm_sensors(
    mut builder: joularcore::monitor::MonitorBuilder,
) -> joularcore::monitor::MonitorBuilder {
    use joularcore::vm::{VmConfig, VmSensor};

    let vm_config = match VmConfig::from_env() {
        Ok(Some(config)) => config,
        Ok(None) => return builder,
        Err(e) => {
            log::warn!("VM monitoring is misconfigured ({e}); using platform monitoring");
            return builder;
        }
    };

    match VmSensor::cpu_from_config(&vm_config) {
        Ok(Some(sensor)) => {
            builder = builder.cpu_sensor(Box::new(sensor));
            log::info!("VM CPU monitoring");
        }
        Ok(None) => {}
        Err(e) => log::warn!("VM CPU monitoring failed ({e}); falling back to platform CPU monitoring"),
    }

    match VmSensor::gpu_from_config(&vm_config) {
        Ok(Some(sensor)) => {
            builder = builder.gpu_sensor(Box::new(sensor));
            log::info!("VM GPU monitoring");
        }
        Ok(None) => {}
        Err(e) => log::warn!("VM GPU monitoring failed ({e}); falling back to platform GPU monitoring"),
    }

    builder
}
