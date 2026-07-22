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

use crate::gui::history::{History, HistoryStats};
use crate::gui::model::MonitorMode;
use crate::gui::theme::tokens;
use crate::gui::{LIVE_BADGE_REPAINT_INTERVAL, PowerGui};
use joularcore::Component;

use eframe::egui;
use eframe::egui::{Color32, FontFamily, FontId, RichText};

const MODE_TITLE_SIZE: f32 = 16.5;
const CARD_LABEL_SIZE: f32 = 12.0;
const CARD_VALUE_SIZE: f32 = 28.0;
const CARD_UNIT_SIZE: f32 = 14.0;
const CARD_STATS_SIZE: f32 = 11.5;

impl PowerGui {
    pub fn draw_monitor(&mut self, ui: &mut egui::Ui) {
        let t = tokens(self.is_dark_mode);

        // Component filter: when the user picked "CPU only" / "GPU only" in
        // the options screen we hide the irrelevant cards entirely and report
        // the total as just the visible component, so the figures the user
        // sees match what gets written to CSV / API / ring buffer.
        let show_cpu = !matches!(self.component_filter, Some(Component::Gpu));
        let show_gpu = !matches!(self.component_filter, Some(Component::Cpu));

        let cpu_power = self.cpu_power;
        let gpu_power = self.gpu_power;
        let total_power = match self.component_filter {
            Some(Component::Cpu) => cpu_power,
            Some(Component::Gpu) => gpu_power,
            None => self.total_power,
        };
        let cpu_usage = self.cpu_usage;
        let process_power = self.process_power;
        let app_power = self.app_power;
        let initialized = self.initialized;
        let monitoring_active = self.monitoring_active;
        let monitor_mode = self.monitor_mode;
        let pid_input = self.pid_input.clone();
        let app_input = self.app_input.clone();

        let pid_label = {
            let pid = pid_input.parse::<u32>().unwrap_or(0);
            if pid > 0 {
                let name = self
                    .pids_list
                    .iter()
                    .find(|(p, _)| *p == pid)
                    .map(|(_, n)| n.as_str())
                    .unwrap_or("Unknown");
                format!("Process {pid} ({name})")
            } else {
                format!("Process {pid_input}")
            }
        };
        let app_label = match app_power {
            Some((_, count)) if !app_input.is_empty() => {
                format!("App: {app_input} ({count} PIDs)")
            }
            _ if !app_input.is_empty() => format!("App: {app_input}"),
            _ => "App Power".to_string(),
        };

        let mode_title = match monitor_mode {
            MonitorMode::Cpu => "Monitoring entire system".to_string(),
            MonitorMode::Pid => format!("Monitoring PID: {}", pid_input),
            MonitorMode::App => format!("Monitoring: {}", app_input),
        };
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                self.draw_top_bar(ui);

                egui::Frame::NONE
                    .inner_margin(egui::Margin {
                        left: 20,
                        right: 20,
                        top: 0,
                        bottom: 0,
                    })
                    .show(ui, |ui| {
                        ui.add_space(18.0);

                        egui::Frame::NONE
                            .fill(t.surface)
                            .stroke(egui::Stroke::new(1.0_f32, t.border))
                            .corner_radius(18.0)
                            .shadow(egui::epaint::Shadow {
                                offset: [0, 10],
                                blur: 24,
                                spread: 0,
                                color: Color32::from_black_alpha(if self.is_dark_mode {
                                    54
                                } else {
                                    18
                                }),
                            })
                            .inner_margin(egui::Margin::same(18))
                            .show(ui, |ui| {
                                ui.vertical_centered(|ui| {
                                    ui.label(
                                        RichText::new(mode_title)
                                            .size(MODE_TITLE_SIZE)
                                            .strong()
                                            .color(t.text_pri),
                                    );
                                    ui.add_space(8.0);
                                    draw_live_badge(ui, monitoring_active, &t);
                                });
                            });

                        ui.add_space(12.0);

                        if monitor_mode == MonitorMode::Pid
                            && let Some(pid_pwr) = process_power
                        {
                            draw_metric_card(
                                ui,
                                &pid_label,
                                pid_pwr,
                                "W",
                                t.accent,
                                &self.process_power_history,
                                initialized,
                                &t,
                            );
                            ui.add_space(10.0);
                        }

                        if monitor_mode == MonitorMode::App
                            && let Some((app_pwr, _)) = app_power
                        {
                            draw_metric_card(
                                ui,
                                &app_label,
                                app_pwr,
                                "W",
                                t.accent,
                                &self.app_power_history,
                                initialized,
                                &t,
                            );
                            ui.add_space(10.0);
                        }

                        match (show_cpu, show_gpu) {
                            // Both components — CPU/GPU on top row, Usage/Total below.
                            (true, true) => {
                                ui.columns(2, |cols| {
                                    draw_metric_card(
                                        &mut cols[0],
                                        "CPU Power",
                                        cpu_power,
                                        "W",
                                        t.cpu,
                                        &self.cpu_power_history,
                                        initialized,
                                        &t,
                                    );
                                    draw_metric_card(
                                        &mut cols[1],
                                        "GPU Power",
                                        gpu_power,
                                        "W",
                                        t.gpu,
                                        &self.gpu_power_history,
                                        initialized,
                                        &t,
                                    );
                                });
                                ui.add_space(10.0);
                                ui.columns(2, |cols| {
                                    draw_metric_card(
                                        &mut cols[0],
                                        "CPU Usage",
                                        cpu_usage,
                                        "%",
                                        t.usage,
                                        &self.cpu_usage_history,
                                        initialized,
                                        &t,
                                    );
                                    draw_metric_card(
                                        &mut cols[1],
                                        "Total Power",
                                        total_power,
                                        "W",
                                        t.power,
                                        &self.total_power_history,
                                        initialized,
                                        &t,
                                    );
                                });
                            }
                            // CPU only — CPU Power + CPU Usage side by side. The Total
                            // card is omitted because Total = CPU Power in this mode.
                            (true, false) => {
                                ui.columns(2, |cols| {
                                    draw_metric_card(
                                        &mut cols[0],
                                        "CPU Power",
                                        cpu_power,
                                        "W",
                                        t.cpu,
                                        &self.cpu_power_history,
                                        initialized,
                                        &t,
                                    );
                                    draw_metric_card(
                                        &mut cols[1],
                                        "CPU Usage",
                                        cpu_usage,
                                        "%",
                                        t.usage,
                                        &self.cpu_usage_history,
                                        initialized,
                                        &t,
                                    );
                                });
                            }
                            // GPU only — just the GPU Power card. CPU Usage and Total
                            // are omitted because they would either be 0 or duplicate
                            // GPU Power.
                            (false, true) => {
                                draw_metric_card(
                                    ui,
                                    "GPU Power",
                                    gpu_power,
                                    "W",
                                    t.gpu,
                                    &self.gpu_power_history,
                                    initialized,
                                    &t,
                                );
                            }
                            // Defensive: filter says hide both. Should never happen
                            // (Component is Cpu | Gpu only), but render nothing rather
                            // than panicking.
                            (false, false) => {}
                        }
                        ui.add_space(18.0);

                        // ── Active-output info row ───────────────────────────
                        let show_rb = self.ringbuffer_committed;
                        #[cfg(feature = "api")]
                        let api_url = if self.api_committed {
                            self.api_port_input
                                .trim()
                                .parse::<u16>()
                                .ok()
                                .map(|p| format!("http://127.0.0.1:{}/data", p))
                        } else {
                            None
                        };
                        #[cfg(not(feature = "api"))]
                        let api_url: Option<String> = None;

                        if show_rb || api_url.is_some() {
                            ui.add_space(4.0);
                            ui.separator();
                            ui.add_space(6.0);

                            if show_rb {
                                let path = joularcore::ringbuffer::RingBufferWriter::shared_path();

                                ui.label(
                                    RichText::new(format!("Ring buffer: {}", path))
                                        .size(11.5)
                                        .color(t.text_sec),
                                );
                            }

                            if let Some(url) = api_url {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("API: ").size(11.5).color(t.text_sec));
                                    ui.add(egui::Hyperlink::from_label_and_url(
                                        RichText::new(&url).size(10.5).color(t.accent),
                                        &url,
                                    ));
                                });
                            }

                            ui.add_space(4.0);
                        }
                    });
            });
    }
}

fn draw_live_badge(ui: &mut egui::Ui, running: bool, t: &crate::gui::theme::Tokens) {
    let (bg_alpha, border_alpha, label, accent, width) = if running {
        (
            if ui.visuals().dark_mode { 38u8 } else { 26u8 },
            110u8,
            "LIVE",
            t.success,
            84.0,
        )
    } else {
        (
            if ui.visuals().dark_mode { 28u8 } else { 20u8 },
            85u8,
            "PAUSED",
            t.text_sec,
            98.0,
        )
    };

    let bg = Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), bg_alpha);
    let border = Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), border_alpha);
    let desired = egui::vec2(width, 26.0);
    let (rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover());
    let painter = ui.painter();

    painter.rect_filled(rect, 13.0, bg);
    painter.rect_stroke(
        rect,
        13.0,
        egui::Stroke::new(1.0_f32, border),
        egui::StrokeKind::Inside,
    );

    let font = FontId::new(12.0, FontFamily::Proportional);
    let dot_radius = 3.5;
    let gap = 8.0;
    let galley = painter.layout_no_wrap(label.to_string(), font.clone(), accent);
    let content_width = dot_radius * 2.0 + gap + galley.size().x;
    let content_left = rect.center().x - content_width / 2.0;
    let dot_center = egui::pos2(content_left + dot_radius, rect.center().y);

    if running {
        let time = ui.ctx().input(|i| i.time);
        let phase = ((time * std::f64::consts::TAU / 0.9).sin() * 0.5 + 0.5) as f32;
        let halo_radius = 5.5 + phase * 2.5;
        let halo_alpha = (80.0 * (1.0 - phase)) as u8;
        let halo_color =
            Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), halo_alpha);
        painter.circle_filled(dot_center, halo_radius, halo_color);
        ui.ctx().request_repaint_after(LIVE_BADGE_REPAINT_INTERVAL);
    }

    painter.circle_filled(dot_center, 3.5, accent);
    let text_pos = egui::pos2(
        dot_center.x + dot_radius + gap,
        rect.center().y - galley.size().y * 0.5,
    );
    painter.galley(text_pos, galley, accent);
}

#[allow(clippy::too_many_arguments)]
fn draw_metric_card(
    ui: &mut egui::Ui,
    title: &str,
    value: f64,
    unit: &str,
    color: Color32,
    history: &History,
    initialized: bool,
    t: &crate::gui::theme::Tokens,
) {
    const CARD_HEIGHT: f32 = 150.0;
    const SPARKLINE_HEIGHT: f32 = 52.0;
    const PAD_X: f32 = 16.0;
    const PAD_TOP: f32 = 16.0;
    const PAD_BOTTOM: f32 = 10.0;
    const STATS_TEXT_HEIGHT: f32 = 12.0;
    const STATS_TO_SPARK_GAP: f32 = 4.0;

    let stats = history.stats();

    egui::Frame::NONE
        .fill(t.elevated)
        .stroke(egui::Stroke::new(1.0_f32, t.border))
        .corner_radius(16.0)
        .shadow(egui::epaint::Shadow {
            offset: [0, 8],
            blur: 18,
            spread: 0,
            color: Color32::from_black_alpha(if ui.visuals().dark_mode { 34 } else { 10 }),
        })
        .inner_margin(egui::Margin::ZERO)
        .show(ui, |ui| {
            let card_width = ui.available_width();
            let size = egui::vec2(card_width, CARD_HEIGHT);
            let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
            let painter = ui.painter();

            let glow_rect = egui::Rect::from_min_max(
                egui::pos2(rect.left(), rect.top()),
                egui::pos2(rect.right(), rect.top() + 35.0),
            );
            painter.rect_filled(
                glow_rect,
                0.0,
                Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 16),
            );

            let dot_center = egui::pos2(rect.left() + PAD_X + 3.5, rect.top() + PAD_TOP + 5.5);
            painter.circle_filled(
                dot_center,
                7.0,
                Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 54),
            );
            painter.circle_filled(dot_center, 3.5, color);

            painter.text(
                egui::pos2(dot_center.x + 8.0, dot_center.y),
                egui::Align2::LEFT_CENTER,
                title.to_uppercase(),
                FontId::new(CARD_LABEL_SIZE, FontFamily::Proportional),
                t.text_sec,
            );

            if initialized {
                let value_text = format!("{value:.1}");
                let value_font = FontId::new(CARD_VALUE_SIZE, FontFamily::Proportional);
                let value_galley = painter.layout_no_wrap(value_text, value_font, color);
                let value_y = rect.top() + PAD_TOP + 20.0;
                let value_pos = egui::pos2(rect.left() + PAD_X, value_y);
                let value_width = value_galley.size().x;
                painter.galley(value_pos, value_galley, color);

                painter.text(
                    egui::pos2(
                        rect.left() + PAD_X + value_width + 4.0,
                        value_y + CARD_VALUE_SIZE - CARD_UNIT_SIZE - 1.0,
                    ),
                    egui::Align2::LEFT_TOP,
                    unit,
                    FontId::new(CARD_UNIT_SIZE, FontFamily::Proportional),
                    t.text_sec,
                );

                let stats_baseline = rect.bottom() - PAD_BOTTOM;
                draw_stats_row(painter, rect.left() + PAD_X, stats_baseline, stats, t);

                if history.len() >= 2 {
                    let spark_bottom = stats_baseline - STATS_TEXT_HEIGHT - STATS_TO_SPARK_GAP;
                    let spark_top = spark_bottom - SPARKLINE_HEIGHT;
                    let spark_rect = egui::Rect::from_min_max(
                        egui::pos2(rect.left(), spark_top),
                        egui::pos2(rect.right(), spark_bottom),
                    );
                    draw_sparkline(painter, spark_rect, history, color);
                }
            } else {
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Loading…",
                    FontId::new(14.0, FontFamily::Proportional),
                    t.text_sec,
                );
            }
        });
}

fn draw_stats_row(
    painter: &egui::Painter,
    start_x: f32,
    baseline_y: f32,
    stats: HistoryStats,
    t: &crate::gui::theme::Tokens,
) {
    let stats_font = FontId::new(CARD_STATS_SIZE, FontFamily::Proportional);
    let segments = [
        format!("min {:.1}", stats.min),
        "·".to_string(),
        format!("avg {:.1}", stats.avg),
        "·".to_string(),
        format!("max {:.1}", stats.max),
    ];
    let mut x = start_x;

    for segment in segments {
        let drawn = painter.text(
            egui::pos2(x, baseline_y),
            egui::Align2::LEFT_BOTTOM,
            segment,
            stats_font.clone(),
            t.text_sec,
        );
        x = drawn.right() + 8.0;
    }
}

fn draw_sparkline(painter: &egui::Painter, rect: egui::Rect, history: &History, color: Color32) {
    if history.len() < 2 {
        return;
    }

    let inner = rect.shrink2(egui::vec2(0.0, 4.0));
    let stats = history.stats();
    let range = (stats.max - stats.min).max(1e-6);
    let len = history.len();
    let mut points = Vec::with_capacity(len);

    for (index, value) in history.iter().enumerate() {
        let x = inner.left() + (index as f32 / (len - 1) as f32) * inner.width();
        let t = ((value - stats.min) / range) as f32;
        let y = inner.bottom() - t * inner.height();
        points.push(egui::pos2(x, y));
    }

    let top_color = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 90);
    let bottom_color = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 5);
    let baseline_y = inner.bottom();

    let mut mesh = egui::epaint::Mesh::default();
    for point in &points {
        mesh.vertices.push(egui::epaint::Vertex {
            pos: *point,
            uv: egui::epaint::WHITE_UV,
            color: top_color,
        });
        mesh.vertices.push(egui::epaint::Vertex {
            pos: egui::pos2(point.x, baseline_y),
            uv: egui::epaint::WHITE_UV,
            color: bottom_color,
        });
    }

    for i in 0..points.len() - 1 {
        let a = (i * 2) as u32;
        mesh.indices
            .extend_from_slice(&[a, a + 1, a + 2, a + 2, a + 1, a + 3]);
    }

    painter.add(egui::Shape::mesh(mesh));
    painter.add(egui::Shape::line(points, egui::Stroke::new(1.8_f32, color)));
}
