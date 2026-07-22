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

use joularcore::Component;
use joularcore::common::{ApiSender, ApiShutdownTx};
use joularcore::monitor::JoularCoreMonitor;
use joularcore::output::{OutputBundle, OutputSink};
use joularcore::ringbuffer::RingBufferWriter;
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

    // Process/App specific values
    pub process_power: Option<f64>,
    pub app_power: Option<(f64, usize)>,

    // Historical data
    pub cpu_power_history: History,
    pub gpu_power_history: History,
    pub total_power_history: History,
    pub cpu_usage_history: History,
    pub process_power_history: History,
    pub app_power_history: History,

    // Energy trackers
    pub monitor: JoularCoreMonitor,

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

    // Display & attribution settings (mirrors -c/--component and --cpu-idle-baseline)
    pub component_filter: Option<Component>,
    pub cpu_idle_baseline_input: String,

    // Ring buffer & API toggles (mirrors -r/--ringbuffer and --api-port).
    // The `*_committed` flags track whether the corresponding resource has
    // already been created during this GUI session — once committed the
    // backing thread / mmap stays for the rest of the run.
    pub ringbuffer_enabled: bool,
    pub ringbuffer_committed: bool,
    pub api_enabled: bool,
    pub api_committed: bool,
    pub api_port_input: String,
    pub api_origin_input: String,
    pub api_allowed_origins: Vec<String>,
    /// Oneshot sender used to gracefully shut down the API server thread.
    pub api_shutdown_tx: ApiShutdownTx,

    // Whether the Advanced card is expanded. Closed by default.
    pub advanced_open: bool,

    // Theme
    pub is_dark_mode: bool,
    theme_state: Option<bool>,
}

impl PowerGui {
    pub fn new(
        monitor: JoularCoreMonitor,
        ringbuffer: Option<RingBufferWriter>,
        api_sender: ApiSender,
        api_shutdown_tx: ApiShutdownTx,
        app_refresh_interval: u64,
        initial_api_port: Option<u16>,
        initial_allowed_origins: Vec<String>,
    ) -> Self {
        let ringbuffer_committed = ringbuffer.is_some();
        let api_committed = api_sender.is_some();
        let api_port_input = initial_api_port.map(|p| p.to_string()).unwrap_or_default();

        let outputs = OutputBundle::new(None, false, ringbuffer, api_sender);

        let mut gui = Self {
            cpu_power: 0.0,
            gpu_power: 0.0,
            total_power: 0.0,
            cpu_usage: 0.0,
            process_power: None,
            app_power: None,

            cpu_power_history: History::new(MAX_HISTORY_LEN),
            gpu_power_history: History::new(MAX_HISTORY_LEN),
            total_power_history: History::new(MAX_HISTORY_LEN),
            cpu_usage_history: History::new(MAX_HISTORY_LEN),
            process_power_history: History::new(MAX_HISTORY_LEN),
            app_power_history: History::new(MAX_HISTORY_LEN),

            monitor,

            state: ValidationState::Options,
            initialized: false,
            last_update: Instant::now(),

            monitor_mode: MonitorMode::Cpu,
            csv_enabled: false,
            csv_path: None,
            csv_overwrite: false,
            system: System::new_all(),
            pids_list: Vec::new(),
            apps_list: Vec::new(),
            app_refresh_interval,

            pid_input: String::new(),
            app_input: String::new(),
            pid_suggestions_open: false,
            app_suggestions_open: false,
            error_message: None,

            outputs,
            monitoring_active: false,

            component_filter: None,
            cpu_idle_baseline_input: String::new(),

            ringbuffer_enabled: ringbuffer_committed,
            ringbuffer_committed,
            api_enabled: api_committed,
            api_committed,
            api_port_input,
            api_origin_input: String::new(),
            api_allowed_origins: initial_allowed_origins,
            api_shutdown_tx,

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
        self.system.processes().values().any(|proc| {
            let name = proc.name().to_string_lossy();
            name == app_name || name.contains(app_name)
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
            // Get and discard the first data
            self.monitor.loop_init();

            // Initialize trackers if selected
            // We can do a dummy poll or just let it naturally happen in next check
            // logic below handles initialization implicitly by calling poll

            self.initialized = true;
            return;
        }

        // Prepare monitor arguments
        let mut pid_arg = None;
        let mut app_arg = None;

        if self.monitor_mode == MonitorMode::Pid {
            if let Ok(pid) = self.pid_input.parse::<u32>() {
                pid_arg = Some(pid);
            }
        } else if self.monitor_mode == MonitorMode::App && !self.app_input.is_empty() {
            app_arg = Some(self.app_input.as_str());
        }

        // Poll monitor — pass through the component filter so the unused
        // sensor (CPU or GPU) isn't read at all in the steady-state loop.
        let sample = self
            .monitor
            .poll(pid_arg, app_arg, self.component_filter.as_ref());

        // Update current readings
        self.cpu_power = sample.cpu_power;
        self.gpu_power = sample.gpu_power;
        self.total_power = sample.total_power;
        self.cpu_usage = sample.cpu_usage;

        // Process/App Power
        self.process_power = sample.process_power;
        self.app_power = sample.app_power; // tuple (power, count)

        // Add to history
        if let Some(p) = self.process_power {
            self.process_power_history.push(p);
        }
        if let Some((p, _)) = self.app_power {
            self.app_power_history.push(p);
        }

        // Update history
        self.cpu_power_history.push(self.cpu_power);
        self.gpu_power_history.push(self.gpu_power);
        self.total_power_history.push(self.total_power);
        self.cpu_usage_history.push(self.cpu_usage);

        if self.monitoring_active {
            let _ = self.outputs.send(&sample);
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

pub fn run_gui(
    monitor: JoularCoreMonitor,
    ringbuffer: Option<RingBufferWriter>,
    api_sender: ApiSender,
    api_shutdown_tx: ApiShutdownTx,
    app_refresh_interval: u64,
    initial_api_port: Option<u16>,
    initial_allowed_origins: Vec<String>,
) -> eframe::Result<()> {
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
            Ok(Box::new(PowerGui::new(
                monitor,
                ringbuffer,
                api_sender,
                api_shutdown_tx,
                app_refresh_interval,
                initial_api_port,
                initial_allowed_origins,
            )))
        }),
    )
}
