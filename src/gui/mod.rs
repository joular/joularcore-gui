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

use crate::args::Args;
use joularcore::monitor::JoularCoreMonitor;
use joularcore::{Component, MonitorConfig, OutputBundle, OutputSink, Target};
pub mod history;
pub mod model;
pub mod theme;
pub mod views;

use eframe::egui;
use history::History;
pub use model::{MonitorMode, ValidationState};
use std::time::{Duration, Instant};
use sysinfo::{Pid, ProcessesToUpdate, System};

const MAX_HISTORY_LEN: usize = 60;
const UPDATE_INTERVAL_SECS: u64 = 1;
pub const UPDATE_INTERVAL: Duration = Duration::from_secs(UPDATE_INTERVAL_SECS);
pub const LIVE_BADGE_REPAINT_INTERVAL: Duration = Duration::from_millis(120);
const PAUSED_REPAINT_INTERVAL: Duration = Duration::from_millis(500);

pub const GUI_WIDTH: f32 = 470.0;
pub const GUI_HEIGHT: f32 = 750.0;
pub const GUI_HEIGHT_EXPANDED: f32 = 750.0;

pub struct PowerGui {
    // Current values
    pub cpu_power: f64,
    pub gpu_power: f64,
    pub total_power: f64,
    pub cpu_usage: f64,

    // Power attributed to the PID or app being targeted, and how many PIDs
    // that app matched at sample time.
    pub target_power: Option<f64>,
    pub app_pid_count: Option<usize>,

    // A sensor that cannot be read reports `None` rather than 0.0, so an
    // unreadable sensor is never mistaken for an idle machine. The gauges still
    // show 0.00 W; these drive the warning banner that explains why.
    pub cpu_unavailable: bool,
    pub gpu_unavailable: bool,

    // Historical data
    pub cpu_power_history: History,
    pub gpu_power_history: History,
    pub total_power_history: History,
    pub cpu_usage_history: History,
    pub process_power_history: History,
    pub app_power_history: History,

    // Energy trackers
    pub monitor: JoularCoreMonitor,

    /// What `monitor` was actually built from.
    pub built_config: MonitorConfig,
    /// What the Options screen is currently asking for. Applied on Start:
    /// target, component and baseline are set on the running monitor, while
    /// `app_match`, `app_refresh_interval` and `elevation` need a new one.
    pub pending_config: MonitorConfig,

    // State
    pub state: ValidationState,
    pub initialized: bool,
    pub last_update: Instant,

    // Options Screen Data
    pub monitor_mode: MonitorMode,
    pub csv_enabled: bool,
    pub csv_path: Option<String>,
    pub csv_overwrite: bool,
    pub system: System,
    pub pids_list: Vec<(u32, String)>,

    pub apps_list: Vec<String>,
    pub app_refresh_interval: u64,

    // Search/Input State
    pub pid_input: String,
    pub app_input: String,
    pub pid_suggestions_open: bool,
    pub app_suggestions_open: bool,
    pub error_message: Option<String>,

    // Outputs
    pub outputs: OutputBundle,
    pub monitoring_active: bool,
    /// A sink that failed mid-session — a CSV file that can no longer be
    /// written, most likely. Shown in the Monitor view.
    pub output_error: Option<String>,

    // Display & attribution settings (mirrors -c/--component and --cpu-idle-baseline)
    pub component_filter: Option<Component>,
    pub cpu_idle_baseline_input: String,

    // Ring buffer toggle (mirrors -r/--ringbuffer). Sinks cannot be detached
    // from an `OutputBundle`, so the bundle is rebuilt on every Start and
    // `ringbuffer_active` simply records whether this session has one.
    pub ringbuffer_enabled: bool,
    pub ringbuffer_active: bool,

    // Whether the Advanced card is expanded. Closed by default.
    pub advanced_open: bool,

    // Theme
    pub is_dark_mode: bool,
    theme_state: Option<bool>,
}

impl PowerGui {
    pub fn new(monitor: JoularCoreMonitor, config: MonitorConfig, args: &Args) -> Self {
        // Command line flags seed the Options screen rather than starting a
        // session: the user still presses Start.
        let (monitor_mode, pid_input, app_input) = match (args.pid, &args.app) {
            (Some(pid), _) => (MonitorMode::Pid, pid.to_string(), String::new()),
            (_, Some(app)) => (MonitorMode::App, String::new(), app.clone()),
            _ => (MonitorMode::Cpu, String::new(), String::new()),
        };

        let mut gui = Self {
            cpu_power: 0.0,
            gpu_power: 0.0,
            total_power: 0.0,
            cpu_usage: 0.0,
            target_power: None,
            app_pid_count: None,
            cpu_unavailable: false,
            gpu_unavailable: false,

            cpu_power_history: History::new(MAX_HISTORY_LEN),
            gpu_power_history: History::new(MAX_HISTORY_LEN),
            total_power_history: History::new(MAX_HISTORY_LEN),
            cpu_usage_history: History::new(MAX_HISTORY_LEN),
            process_power_history: History::new(MAX_HISTORY_LEN),
            app_power_history: History::new(MAX_HISTORY_LEN),

            monitor,

            built_config: config.clone(),

            state: ValidationState::Options,
            initialized: false,
            last_update: Instant::now(),

            monitor_mode,
            csv_enabled: args.file.is_some(),
            csv_path: args.file.clone(),
            csv_overwrite: args.overwrite,
            system: System::new_all(),
            pids_list: Vec::new(),
            apps_list: Vec::new(),
            app_refresh_interval: config.app_refresh_interval.as_secs(),

            pid_input,
            app_input,
            pid_suggestions_open: false,
            app_suggestions_open: false,
            error_message: None,

            outputs: OutputBundle::new(),
            monitoring_active: false,
            output_error: None,

            component_filter: config.component,
            cpu_idle_baseline_input: config
                .cpu_idle_baseline
                .map(|b| format!("{b:.2}"))
                .unwrap_or_default(),

            ringbuffer_enabled: args.ringbuffer,
            ringbuffer_active: false,

            pending_config: config,

            advanced_open: false,

            is_dark_mode: false, // Default to Light Mode
            theme_state: None,
        };

        gui.refresh_processes();
        gui
    }

    fn apply_theme(&mut self, ctx: &egui::Context) {
        if self.theme_state == Some(self.is_dark_mode) {
            return;
        }

        theme::apply_theme(ctx, self.is_dark_mode);
        self.theme_state = Some(self.is_dark_mode);
    }

    fn refresh_processes(&mut self) {
        self.system.refresh_processes(ProcessesToUpdate::All, true);

        // PIDs
        self.pids_list = self
            .system
            .processes()
            .iter()
            .map(|(pid, proc)| (pid.as_u32(), proc.name().to_string_lossy().to_string()))
            .collect();
        self.pids_list.sort_by(|a, b| a.1.cmp(&b.1));

        // Apps (unique names)
        let mut apps: Vec<String> = self
            .pids_list
            .iter()
            .map(|(_, name)| name.clone())
            .collect();
        apps.sort();
        apps.dedup();
        self.apps_list = apps;
    }

    fn pid_exists(&mut self, pid: u32) -> bool {
        self.system.refresh_processes(ProcessesToUpdate::All, true);
        self.system.processes().contains_key(&Pid::from_u32(pid))
    }

    fn app_exists(&mut self, app_name: &str) -> bool {
        self.system.refresh_processes(ProcessesToUpdate::All, true);
        let app_match = self.pending_config.app_match;
        // Matched the way the library will match it while measuring, so a
        // session that starts is one that can actually attribute power.
        self.system.processes().values().any(|proc| {
            if proc.thread_kind().is_some() {
                return false;
            }
            crate::args::matches_app_name(&proc.name().to_string_lossy(), app_name, app_match)
        })
    }

    pub fn reset_sampling_baseline(&mut self) {
        // Force one warm-up cycle before the next visible sample so we don't
        // report power accumulated during idle/paused periods.
        self.initialized = false;
        self.last_update = Instant::now();
    }

    fn update_values(&mut self) {
        if !self.initialized {
            // Power and utilization are both counter deltas, so the first
            // reading only establishes a baseline. Take and discard it.
            self.monitor.prime();

            // Starting a session creates a tracker with no history either, and
            // its first attribution sample is a baseline in the same way. Burn
            // it here so the first figure the user sees is a real one rather
            // than 0.00 W.
            if !matches!(self.built_config.target, Target::System) {
                self.monitor.poll();
            }

            self.initialized = true;
            return;
        }

        // Target and component are state on the monitor, set once when the
        // session started — the sensor a component filter excludes is not read.
        let sample = self.monitor.poll();

        // A sensor that could not be read is `None`, not 0.0. The gauges show
        // 0.00 W either way; the flags are what let the Monitor view say why.
        self.cpu_unavailable = sample.cpu_power.is_none();
        self.gpu_unavailable = sample.gpu_power.is_none();

        // Update current readings
        self.cpu_power = sample.cpu_power_or_zero();
        self.gpu_power = sample.gpu_power_or_zero();
        self.total_power = sample.total_power();
        self.cpu_usage = sample.cpu_usage;

        // Power attributed to the PID or app being targeted
        self.target_power = sample.target_power;
        self.app_pid_count = sample.app_pid_count;

        // Add to history
        if let Some(p) = self.target_power {
            match self.monitor_mode {
                MonitorMode::Pid => self.process_power_history.push(p),
                MonitorMode::App => self.app_power_history.push(p),
                MonitorMode::Cpu => {}
            }
        }

        // Update history
        self.cpu_power_history.push(self.cpu_power);
        self.gpu_power_history.push(self.gpu_power);
        self.total_power_history.push(self.total_power);
        self.cpu_usage_history.push(self.cpu_usage);

        if self.monitoring_active
            && let Err(e) = self.outputs.send(&sample)
        {
            // A CSV file that can no longer be written would otherwise fail
            // silently for the rest of the session.
            self.output_error = Some(e.to_string());
            self.monitoring_active = false;
        }
    }
}

impl eframe::App for PowerGui {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.state == ValidationState::Monitor && self.monitoring_active {
            // Update values periodically
            if self.last_update.elapsed() >= UPDATE_INTERVAL {
                self.update_values();
                self.last_update = Instant::now();
            }

            let until_next_sample = UPDATE_INTERVAL.saturating_sub(self.last_update.elapsed());
            ctx.request_repaint_after(until_next_sample);
        } else if self.state == ValidationState::Monitor && !self.monitoring_active {
            ctx.request_repaint_after(PAUSED_REPAINT_INTERVAL);
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.apply_theme(ui.ctx());

        egui::CentralPanel::default().show(ui, |ui| match self.state {
            ValidationState::Options => self.draw_options(ui),
            ValidationState::Monitor => self.draw_monitor(ui),
        });
    }
}

pub fn run_gui(monitor: JoularCoreMonitor, config: MonitorConfig, args: Args) -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([GUI_WIDTH, GUI_HEIGHT])
            // Don't let the user shrink the window below the layout's design
            // width — cards and the sparkline start clipping otherwise.
            .with_min_inner_size([GUI_WIDTH, 480.0])
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "Joular Core",
        native_options,
        Box::new(move |cc| {
            theme::install_fonts(&cc.egui_ctx);
            Ok(Box::new(PowerGui::new(monitor, config, &args)))
        }),
    )
}
