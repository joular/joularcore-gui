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

use crate::gui::PowerGui;
use crate::gui::model::MonitorMode;
use crate::gui::model::ValidationState;
use crate::gui::theme::tokens;
use joularcore::Component;
use joularcore::output::OutputWriter;

use eframe::egui;
use eframe::egui::{Color32, RichText};
use rfd::FileDialog;
use std::f32::consts::PI;

const ROW_HEIGHT: f32 = 30.0;
const BODY_TEXT_SIZE: f32 = 13.5;
const SUPPORT_TEXT_SIZE: f32 = 12.5;
const FOOTER_TEXT_SIZE: f32 = 11.5;
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Copy, Clone)]
enum UiIcon {
    Refresh,
    List,
    Search,
}

fn paint_icon(painter: &egui::Painter, rect: egui::Rect, icon: UiIcon, color: Color32) {
    let stroke = egui::Stroke::new(1.8_f32, color);
    let center = rect.center();

    match icon {
        UiIcon::Refresh => {
            let radius = rect.width().min(rect.height()) * 0.22;
            let start = -PI * 0.25;
            let end = PI * 1.25;
            let steps = 36;
            let mut points = Vec::with_capacity(steps + 1);

            for step in 0..=steps {
                let t = step as f32 / steps as f32;
                let angle = start + (end - start) * t;
                points.push(center + radius * egui::vec2(angle.cos(), angle.sin()));
            }

            painter.add(egui::Shape::line(points, stroke));

            let tip_angle = end;
            let tip = center + radius * egui::vec2(tip_angle.cos(), tip_angle.sin());

            // Use a point slightly back along the arc to get a secant direction.
            // This visually centers the arrowhead on the curved line.
            let tail_angle = tip_angle - 0.4;
            let tail = center + radius * egui::vec2(tail_angle.cos(), tail_angle.sin());
            let dir = (tip - tail).normalized();
            let perp = egui::vec2(-dir.y, dir.x);

            let arrow_len = 4.0;
            let arrow_width = 3.2;
            let p1 = tip - dir * arrow_len + perp * arrow_width;
            let p2 = tip - dir * arrow_len - perp * arrow_width;

            painter.line_segment([tip, p1], stroke);
            painter.line_segment([tip, p2], stroke);
        }
        UiIcon::List => {
            let left = rect.left() + 8.0;
            let right = rect.right() - 8.0;
            let offsets = [-5.0, 0.0, 5.0];

            for offset in offsets {
                let y = center.y + offset;
                painter.line_segment([egui::pos2(left, y), egui::pos2(right, y)], stroke);
            }
        }
        UiIcon::Search => {
            let radius = rect.width().min(rect.height()) * 0.22;
            let lens_center = center + egui::vec2(-2.0, -1.5);
            painter.circle_stroke(lens_center, radius, stroke);

            let handle_start = lens_center + egui::vec2(radius * 0.65, radius * 0.65);
            let handle_end = handle_start + egui::vec2(5.0, 5.0);
            painter.line_segment([handle_start, handle_end], stroke);
        }
    }
}

fn add_icon_button(ui: &mut egui::Ui, icon: UiIcon) -> egui::Response {
    let desired_size = egui::vec2(ROW_HEIGHT, ROW_HEIGHT);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let t = tokens(ui.visuals().dark_mode);
        let fill = if response.hovered() {
            t.overlay
        } else {
            t.surface
        };

        ui.painter().rect(
            rect,
            10.0,
            fill,
            egui::Stroke::new(1.0_f32, t.border),
            egui::StrokeKind::Inside,
        );
        paint_icon(ui.painter(), rect, icon, t.text_sec);
    }

    response
}

fn styled_text_input(
    text: &mut String,
    hint: &'static str,
    ui: &mut egui::Ui,
    width: f32,
) -> egui::Response {
    let t = tokens(ui.visuals().dark_mode);
    let stroke = egui::Stroke::new(1.0_f32, t.border);

    ui.scope(|ui| {
        ui.visuals_mut().widgets.inactive.bg_fill = t.surface;
        ui.visuals_mut().widgets.inactive.weak_bg_fill = t.surface;
        ui.visuals_mut().widgets.hovered.bg_fill = t.overlay;
        ui.visuals_mut().widgets.hovered.weak_bg_fill = t.overlay;
        ui.visuals_mut().widgets.active.bg_fill = t.overlay;
        ui.visuals_mut().widgets.active.weak_bg_fill = t.overlay;
        ui.visuals_mut().widgets.inactive.bg_stroke = stroke;
        ui.visuals_mut().widgets.hovered.bg_stroke = stroke;
        ui.visuals_mut().widgets.active.bg_stroke = stroke;
        ui.visuals_mut().widgets.open.bg_stroke = stroke;
        ui.visuals_mut().selection.stroke = stroke;
        ui.visuals_mut().widgets.inactive.corner_radius = 10.0.into();
        ui.visuals_mut().widgets.hovered.corner_radius = 10.0.into();
        ui.visuals_mut().widgets.active.corner_radius = 10.0.into();
        ui.visuals_mut().widgets.open.corner_radius = 10.0.into();

        let response = ui.add_sized(
            [width, ROW_HEIGHT],
            egui::TextEdit::singleline(text)
                .hint_text(hint)
                .margin(egui::Margin {
                    left: 24,
                    right: 5,
                    top: 5,
                    bottom: 5,
                }),
        );

        if ui.is_rect_visible(response.rect) {
            let icon_rect = egui::Rect::from_center_size(
                egui::pos2(response.rect.left() + 13.0, response.rect.center().y),
                egui::vec2(16.0, 16.0),
            );
            paint_icon(ui.painter(), icon_rect, UiIcon::Search, t.text_ter);
        }

        response
    })
    .inner
}

fn card_frame(t: &crate::gui::theme::Tokens) -> egui::Frame {
    egui::Frame::NONE
        .fill(t.elevated)
        .stroke(egui::Stroke::new(1.0_f32, t.border))
        .corner_radius(14.0)
        .shadow(egui::epaint::Shadow {
            offset: [0, 8],
            blur: 18,
            spread: 0,
            color: Color32::from_black_alpha(14),
        })
        .inner_margin(egui::Margin {
            left: 16,
            right: 16,
            top: 15,
            bottom: 15,
        })
}

impl PowerGui {
    pub fn draw_options(&mut self, ui: &mut egui::Ui) {
        let t = tokens(self.is_dark_mode);

        egui::ScrollArea::vertical().show(ui, |ui| {
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

                    self.draw_tab_panel(ui, &t);

                    ui.add_space(16.0);

                    // ── CSV Logging card (expands to include browse + overwrite) ─
                    card_frame(&t).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new("CSV Logging")
                                        .size(BODY_TEXT_SIZE)
                                        .strong()
                                        .color(t.text_pri),
                                );
                                ui.add_space(2.0);
                                ui.label(
                                    RichText::new("Save metrics to a .csv file")
                                        .size(12.5)
                                        .color(t.text_sec),
                                );
                            });
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    PowerGui::toggle_ui(ui, &mut self.csv_enabled);
                                },
                            );
                        });

                        if self.csv_enabled {
                            ui.add_space(12.0);
                            // In-card divider
                            let sep_y = ui.cursor().top();
                            ui.painter().line_segment(
                                [
                                    egui::pos2(ui.min_rect().left(), sep_y),
                                    egui::pos2(ui.min_rect().right(), sep_y),
                                ],
                                egui::Stroke::new(1.0_f32, t.border),
                            );
                            ui.add_space(12.0);

                            ui.horizontal(|ui| {
                                if ui
                                    .add(
                                        egui::Button::new(
                                            RichText::new("Browse…")
                                                .size(SUPPORT_TEXT_SIZE)
                                                .color(t.text_pri),
                                        )
                                        .fill(t.surface)
                                        .stroke(egui::Stroke::new(1.0_f32, t.border))
                                        .corner_radius(10.0)
                                        .min_size(egui::vec2(88.0, 30.0)),
                                    )
                                    .clicked()
                                    && let Some(path) = FileDialog::new().save_file()
                                {
                                    self.csv_path = Some(path.display().to_string());
                                }
                                ui.label(
                                    RichText::new(
                                        self.csv_path.as_deref().unwrap_or("No file selected"),
                                    )
                                    .size(SUPPORT_TEXT_SIZE)
                                    .color(
                                        if self.csv_path.is_none() {
                                            t.text_ter
                                        } else {
                                            t.text_sec
                                        },
                                    ),
                                );
                            });
                            ui.add_space(10.0);
                            ui.horizontal(|ui| {
                                PowerGui::toggle_ui(ui, &mut self.csv_overwrite);
                                ui.add_space(8.0);
                                ui.label(
                                    RichText::new("Overwrite existing file")
                                        .size(SUPPORT_TEXT_SIZE)
                                        .color(t.text_pri),
                                );
                            });
                        }
                    });

                    ui.add_space(16.0);

                    // ── Advanced card (collapsible, closed by default) ───────
                    card_frame(&t).show(ui, |ui| {
                        // Header row: "Advanced" + chevron, whole row clickable.
                        let header_row_size = egui::vec2(ui.available_width(), 22.0);
                        let (header_rect, header_resp) =
                            ui.allocate_exact_size(header_row_size, egui::Sense::click());
                        if header_resp.clicked() {
                            self.advanced_open = !self.advanced_open;
                        }
                        let chevron = if self.advanced_open { "v" } else { ">" };
                        ui.painter().text(
                            egui::pos2(header_rect.left(), header_rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            chevron,
                            egui::FontId::proportional(13.0),
                            t.text_sec,
                        );
                        ui.painter().text(
                            egui::pos2(header_rect.left() + 16.0, header_rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            "Advanced",
                            egui::FontId::proportional(BODY_TEXT_SIZE),
                            t.text_pri,
                        );
                        ui.painter().text(
                            egui::pos2(header_rect.right(), header_rect.center().y),
                            egui::Align2::RIGHT_CENTER,
                            if self.advanced_open {
                                "Hide"
                            } else {
                                "Configure"
                            },
                            egui::FontId::proportional(11.5),
                            t.text_sec,
                        );

                        if !self.advanced_open {
                            return;
                        }

                        ui.add_space(10.0);
                        ui.painter().line_segment(
                            [
                                egui::pos2(ui.min_rect().left(), ui.cursor().top()),
                                egui::pos2(ui.min_rect().right(), ui.cursor().top()),
                            ],
                            egui::Stroke::new(1.0_f32, t.border),
                        );
                        ui.add_space(10.0);

                        // ── Component filter: Both / CPU only / GPU only ─────
                        ui.label(
                            RichText::new("Component")
                                .size(SUPPORT_TEXT_SIZE)
                                .color(t.text_pri),
                        );
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            for (label, value) in [
                                ("Both", None),
                                ("CPU only", Some(Component::Cpu)),
                                ("GPU only", Some(Component::Gpu)),
                            ] {
                                let selected = self.component_filter == value;
                                let fill = if selected { t.accent } else { t.surface };
                                let text_color = if selected { Color32::WHITE } else { t.text_pri };
                                if ui
                                    .add(
                                        egui::Button::new(
                                            RichText::new(label)
                                                .size(SUPPORT_TEXT_SIZE)
                                                .color(text_color),
                                        )
                                        .fill(fill)
                                        .stroke(egui::Stroke::new(1.0_f32, t.border))
                                        .corner_radius(10.0)
                                        .min_size(egui::vec2(96.0, 28.0)),
                                    )
                                    .clicked()
                                {
                                    self.component_filter = value;
                                }
                            }
                        });

                        ui.add_space(14.0);

                        // ── CPU idle baseline (watts) ────────────────────────
                        ui.label(
                            RichText::new("CPU idle baseline (W)")
                                .size(SUPPORT_TEXT_SIZE)
                                .color(t.text_pri),
                        );
                        ui.add_space(4.0);
                        styled_text_input(
                            &mut self.cpu_idle_baseline_input,
                            "e.g. 4.5  (leave blank to keep current)",
                            ui,
                            ui.available_width(),
                        );
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(
                                "Subtracted from CPU power before attributing it to a PID or app.",
                            )
                            .size(12.0)
                            .color(t.text_ter),
                        );

                        ui.add_space(14.0);

                        // ── Ring buffer toggle ───────────────────────────────
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new("Ring buffer")
                                        .size(SUPPORT_TEXT_SIZE)
                                        .color(t.text_pri),
                                );
                                ui.label(
                                    RichText::new(
                                        "Write samples to shared memory for low-latency IPC.",
                                    )
                                    .size(12.0)
                                    .color(t.text_ter),
                                );
                            });
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let mut toggle = self.ringbuffer_enabled;
                                    let response = PowerGui::toggle_ui(ui, &mut toggle);
                                    if response.changed() {
                                        self.ringbuffer_enabled = toggle;
                                        if !toggle && self.ringbuffer_committed {
                                            self.outputs.set_ringbuffer(None);
                                            self.ringbuffer_committed = false;
                                        }
                                    }
                                },
                            );
                        });
                        if self.ringbuffer_committed {
                            ui.label(
                                RichText::new("(active)")
                                    .size(11.5)
                                    .italics()
                                    .color(t.text_ter),
                            );
                        }

                        ui.add_space(14.0);

                        // ── HTTP / WebSocket API toggle + port + origins ─────
                        #[cfg(feature = "api")]
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new("HTTP / WebSocket API")
                                        .size(SUPPORT_TEXT_SIZE)
                                        .color(t.text_pri),
                                );
                                ui.label(
                                    RichText::new(
                                        "Expose live samples on /data and /ws (localhost only).",
                                    )
                                    .size(12.0)
                                    .color(t.text_ter),
                                );
                            });
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let mut toggle = self.api_enabled;
                                    let response = PowerGui::toggle_ui(ui, &mut toggle);
                                    if response.changed() {
                                        self.api_enabled = toggle;
                                        if !toggle && self.api_committed {
                                            if let Some(tx) = self.api_shutdown_tx.take() {
                                                let _ = tx.send(());
                                            }
                                            self.outputs.set_api_sender(None);
                                            self.api_committed = false;
                                        }
                                    }
                                },
                            );
                        });

                        #[cfg(feature = "api")]
                        if self.api_enabled {
                            ui.add_space(8.0);
                            ui.label(
                                RichText::new("Port")
                                    .size(SUPPORT_TEXT_SIZE)
                                    .color(t.text_pri),
                            );
                            ui.add_space(4.0);
                            styled_text_input(
                                &mut self.api_port_input,
                                "e.g. 8080",
                                ui,
                                ui.available_width(),
                            );

                            ui.add_space(10.0);
                            ui.label(
                                RichText::new("Allowed origins (extra)")
                                    .size(SUPPORT_TEXT_SIZE)
                                    .color(t.text_pri),
                            );
                            ui.label(
                                RichText::new(
                                    "Localhost is always allowed. Add more if a remote dashboard needs CORS access.",
                                )
                                .size(11.5)
                                .color(t.text_ter),
                            );
                            ui.add_space(4.0);

                            // Add-origin row.
                            let mut origin_to_add: Option<String> = None;
                            ui.horizontal(|ui| {
                                let btn_w = 60.0;
                                let input_w =
                                    (ui.available_width() - btn_w - ui.spacing().item_spacing.x)
                                        .max(60.0);
                                let _resp = styled_text_input(
                                    &mut self.api_origin_input,
                                    "https://my-dashboard.example.com",
                                    ui,
                                    input_w,
                                );
                                if ui
                                    .add(
                                        egui::Button::new(
                                            RichText::new("Add")
                                                .size(SUPPORT_TEXT_SIZE)
                                                .color(t.text_pri),
                                        )
                                        .fill(t.surface)
                                        .stroke(egui::Stroke::new(1.0_f32, t.border))
                                        .corner_radius(10.0)
                                        .min_size(egui::vec2(btn_w, 30.0)),
                                    )
                                    .clicked()
                                    && !self.api_committed
                                {
                                    let trimmed = self.api_origin_input.trim().to_string();
                                    if !trimmed.is_empty()
                                        && !self.api_allowed_origins.contains(&trimmed)
                                    {
                                        origin_to_add = Some(trimmed);
                                    }
                                    self.api_origin_input.clear();
                                }
                            });
                            if let Some(o) = origin_to_add {
                                self.api_allowed_origins.push(o);
                            }

                            // Existing origins list with remove buttons.
                            if self.api_allowed_origins.is_empty() {
                                ui.add_space(4.0);
                                ui.label(
                                    RichText::new("(no extra origins)")
                                        .italics()
                                        .size(12.0)
                                        .color(t.text_ter),
                                );
                            } else {
                                ui.add_space(4.0);
                                let mut remove_idx: Option<usize> = None;
                                for (i, origin) in self.api_allowed_origins.iter().enumerate() {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new(origin)
                                                .size(SUPPORT_TEXT_SIZE)
                                                .color(t.text_sec),
                                        );
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if !self.api_committed
                                                    && ui
                                                        .add(
                                                            egui::Button::new(
                                                                RichText::new("Remove")
                                                                    .size(11.0)
                                                                    .color(t.danger),
                                                            )
                                                            .fill(t.surface)
                                                            .stroke(egui::Stroke::new(
                                                                1.0_f32, t.border,
                                                            ))
                                                            .corner_radius(8.0),
                                                        )
                                                        .clicked()
                                                {
                                                    remove_idx = Some(i);
                                                }
                                            },
                                        );
                                    });
                                }
                                if let Some(i) = remove_idx {
                                    self.api_allowed_origins.remove(i);
                                }
                            }

                            if self.api_committed {
                                ui.add_space(6.0);
                                ui.label(
                                    RichText::new(
                                        "(active — port and origins are locked until the server is restarted)",
                                    )
                                    .size(11.5)
                                    .italics()
                                    .color(t.text_ter),
                                );
                            }
                        }
                    });

                    ui.add_space(20.0);

                    // ── Error + Start button ─────────────────────────────────
                    if let Some(message) = &self.error_message {
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new(message)
                                    .color(t.danger)
                                    .size(SUPPORT_TEXT_SIZE),
                            );
                        });
                        ui.add_space(8.0);
                    }

                    // Start button (roughly half width, centered)
                    ui.horizontal(|ui| {
                        let avail = ui.available_width();
                        let btn_w = avail * 0.5;
                        let pad = (avail - btn_w) * 0.5;
                        ui.add_space(pad);

                        let (btn_rect, btn_response) =
                            ui.allocate_exact_size(egui::vec2(btn_w, 46.0), egui::Sense::click());
                        let shadow_color = Color32::from_rgba_unmultiplied(
                            t.accent.r(),
                            t.accent.g(),
                            t.accent.b(),
                            70,
                        );
                        let halo = btn_rect.expand(2.0).translate(egui::vec2(0.0, 5.0));
                        ui.painter().rect_filled(halo, 14.0, shadow_color);
                        let btn_fill = if btn_response.is_pointer_button_down_on() {
                            t.accent.linear_multiply(0.85)
                        } else {
                            t.accent
                        };
                        ui.painter().rect_filled(btn_rect, 14.0, btn_fill);
                        ui.painter().text(
                            btn_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "▶  Start Monitoring",
                            egui::FontId::new(14.5, egui::FontFamily::Proportional),
                            Color32::WHITE,
                        );
                        if btn_response.clicked() {
                            self.handle_start_clicked();
                        }
                    });

                    ui.add_space(18.0);

                    // ── Footer ──────────────────────────────────────────────
                    ui.painter().line_segment(
                        [
                            egui::pos2(ui.min_rect().left(), ui.cursor().top()),
                            egui::pos2(ui.min_rect().right(), ui.cursor().top()),
                        ],
                        egui::Stroke::new(1.0_f32, t.border),
                    );
                    ui.add_space(10.0);

                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new(format!(
                                "Platform: {} · Version: {}",
                                self.monitor.platform.name(),
                                APP_VERSION
                            ))
                            .size(11.5)
                            .color(t.text_ter),
                        );
                        ui.label(
                            RichText::new("Joular Core is a platform to measure power and energy across all systems, OSes and devices. Joular Core is licensed under the GNU GPL 3 license only (GPL-3.0-only).\nCopyright © 2025-2026, Adel Noureddine. All rights reserved.")
                                .size(FOOTER_TEXT_SIZE)
                                .color(t.text_ter),
                        );
                        ui.hyperlink_to(
                            RichText::new("github.com/joular/joularcore")
                                .size(11.0)
                                .color(t.accent),
                            "https://github.com/joular/joularcore",
                        );
                    });
                    ui.add_space(14.0);
                });
        });
    }

    fn draw_tab_panel(&mut self, ui: &mut egui::Ui, t: &crate::gui::theme::Tokens) {
        const TAB_HEIGHT: f32 = 38.0;

        egui::Frame::NONE
            .fill(t.surface)
            .stroke(egui::Stroke::new(1.0_f32, t.border))
            .corner_radius(16.0)
            .inner_margin(egui::Margin::ZERO)
            .show(ui, |ui| {
                let full_w = ui.available_width();
                let (row_rect, _) = ui.allocate_exact_size(
                    egui::vec2(full_w, TAB_HEIGHT),
                    egui::Sense::hover(),
                );

                ui.painter().rect_filled(row_rect.shrink(3.0), 12.0, t.elevated);

                let modes = [
                    (MonitorMode::Cpu, "Entire System"),
                    (MonitorMode::Pid, "Process (PID)"),
                    (MonitorMode::App, "Application"),
                ];
                let tab_w = full_w / modes.len() as f32;

                for (i, (mode, label)) in modes.iter().enumerate() {
                    let tab_rect = egui::Rect::from_min_size(
                        egui::pos2(row_rect.left() + i as f32 * tab_w, row_rect.top()),
                        egui::vec2(tab_w, TAB_HEIGHT),
                    );
                    let id = ui.id().with(("monitor_mode_tab", i));
                    let response = ui.interact(tab_rect, id, egui::Sense::click());
                    let selected = self.monitor_mode == *mode;

                    if response.hovered() && !selected {
                        ui.painter().rect_filled(
                            tab_rect.shrink2(egui::vec2(4.0, 3.0)),
                            10.0,
                            t.overlay,
                        );
                    }

                    if selected {
                        let active_rect = tab_rect.shrink2(egui::vec2(4.0, 3.0));
                        ui.painter().rect_filled(active_rect, 10.0, t.accent);
                    }

                    let text_color = if selected { Color32::WHITE } else { t.text_sec };
                    ui.painter().text(
                        tab_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        label,
                        egui::FontId::new(12.5, egui::FontFamily::Proportional),
                        text_color,
                    );

                    if response.clicked() {
                        self.monitor_mode = *mode;
                        self.error_message = None;
                    }
                }

                egui::Frame::NONE
                    .inner_margin(egui::Margin::symmetric(16, 16))
                    .show(ui, |ui| {
                        let desc = match self.monitor_mode {
                            MonitorMode::Cpu => {
                                "Measures total CPU and GPU power consumption across the entire system."
                            }
                            MonitorMode::Pid => {
                                "Tracks energy usage of a specific process by its PID."
                            }
                            MonitorMode::App => {
                                "Monitors power draw from a named application or bundle."
                            }
                        };
                        ui.label(
                            RichText::new(desc)
                                .size(SUPPORT_TEXT_SIZE)
                                .color(t.text_sec),
                        );

                        match self.monitor_mode {
                            MonitorMode::Cpu => {}
                            MonitorMode::Pid => {
                                ui.add_space(10.0);
                                self.draw_pid_section(ui, t);
                            }
                            MonitorMode::App => {
                                ui.add_space(10.0);
                                self.draw_app_section(ui, t);
                            }
                        }
                    });
            });
    }

    fn draw_pid_section(&mut self, ui: &mut egui::Ui, t: &crate::gui::theme::Tokens) {
        ui.horizontal(|ui| {
            let btn_total = 2.0 * ROW_HEIGHT + 2.0 * ui.spacing().item_spacing.x;
            let input_w = (ui.available_width() - btn_total).max(60.0);

            let response = styled_text_input(
                &mut self.pid_input,
                "Search by PID or process name…",
                ui,
                input_w,
            );
            if response.changed() {
                self.error_message = None;
                self.pid_suggestions_open = true;
                if self.pid_input.is_empty() {
                    self.pid_suggestions_open = false;
                }
            }

            if add_icon_button(ui, UiIcon::Refresh)
                .on_hover_text("Refresh process list")
                .clicked()
            {
                self.refresh_processes();
            }
            if add_icon_button(ui, UiIcon::List)
                .on_hover_text("Show / hide process list")
                .clicked()
            {
                self.pid_suggestions_open = !self.pid_suggestions_open;
            }
        });

        if self.pid_suggestions_open {
            ui.add_space(6.0);
            egui::Frame::NONE
                .fill(t.surface)
                .stroke(egui::Stroke::new(1.0_f32, t.border))
                .inner_margin(10.0)
                .corner_radius(12.0)
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .show(ui, |ui| {
                            let filter = self.pid_input.to_lowercase();
                            let mut count = 0;
                            for (pid, name) in &self.pids_list {
                                let s = format!("{} {}", pid, name).to_lowercase();
                                if filter.is_empty() || s.contains(&filter) {
                                    if ui
                                        .add_sized(
                                            [ui.available_width(), 28.0],
                                            egui::Button::new(
                                                RichText::new(format!("{} — {}", pid, name))
                                                    .size(SUPPORT_TEXT_SIZE)
                                                    .color(t.text_pri),
                                            )
                                            .fill(Color32::TRANSPARENT),
                                        )
                                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                                        .clicked()
                                    {
                                        self.pid_input = pid.to_string();
                                        self.pid_suggestions_open = false;
                                    }
                                    count += 1;
                                }
                            }
                            if count == 0 {
                                ui.label(
                                    RichText::new("No matching processes found")
                                        .italics()
                                        .color(t.text_sec),
                                );
                            }
                        });
                });
        }
    }

    fn draw_app_section(&mut self, ui: &mut egui::Ui, t: &crate::gui::theme::Tokens) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("PID refresh interval:")
                    .size(SUPPORT_TEXT_SIZE)
                    .color(t.text_sec),
            );

            let res = ui
                .scope(|ui| {
                    ui.visuals_mut().widgets.inactive.bg_fill = t.overlay;
                    ui.visuals_mut().widgets.active.bg_fill = t.accent;
                    ui.visuals_mut().widgets.hovered.bg_fill = t.overlay;
                    ui.add_sized(
                        [110.0, 18.0],
                        egui::Slider::new(&mut self.app_refresh_interval, 0..=10).show_value(false),
                    )
                })
                .inner;

            if res.changed()
                && let Some(tracker) = &mut self.monitor.app_tracker
            {
                tracker.set_refresh_interval(std::time::Duration::from_secs(
                    self.app_refresh_interval,
                ));
            }

            let badge_text = if self.app_refresh_interval == 0 {
                "off".to_string()
            } else {
                format!("{}s", self.app_refresh_interval)
            };
            ui.label(
                RichText::new(badge_text)
                    .size(SUPPORT_TEXT_SIZE)
                    .strong()
                    .color(t.accent),
            );
        });

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            let btn_total = 2.0 * ROW_HEIGHT + 2.0 * ui.spacing().item_spacing.x;
            let input_w = (ui.available_width() - btn_total).max(60.0);

            let response = styled_text_input(
                &mut self.app_input,
                "Search by application name…",
                ui,
                input_w,
            );
            if response.changed() {
                self.error_message = None;
                self.app_suggestions_open = true;
                if self.app_input.is_empty() {
                    self.app_suggestions_open = false;
                }
            }

            if add_icon_button(ui, UiIcon::Refresh)
                .on_hover_text("Refresh application list")
                .clicked()
            {
                self.refresh_processes();
            }
            if add_icon_button(ui, UiIcon::List)
                .on_hover_text("Show / hide application list")
                .clicked()
            {
                self.app_suggestions_open = !self.app_suggestions_open;
            }
        });

        if self.app_suggestions_open {
            ui.add_space(6.0);
            egui::Frame::NONE
                .fill(t.surface)
                .stroke(egui::Stroke::new(1.0_f32, t.border))
                .inner_margin(10.0)
                .corner_radius(12.0)
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .show(ui, |ui| {
                            let filter = self.app_input.to_lowercase();
                            let mut count = 0;
                            for name in &self.apps_list {
                                if filter.is_empty() || name.to_lowercase().contains(&filter) {
                                    if ui
                                        .add_sized(
                                            [ui.available_width(), 28.0],
                                            egui::Button::new(
                                                RichText::new(name)
                                                    .size(SUPPORT_TEXT_SIZE)
                                                    .color(t.text_pri),
                                            )
                                            .fill(Color32::TRANSPARENT),
                                        )
                                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                                        .clicked()
                                    {
                                        self.app_input = name.clone();
                                        self.app_suggestions_open = false;
                                    }
                                    count += 1;
                                }
                            }
                            if count == 0 {
                                ui.label(
                                    RichText::new("No matching apps found")
                                        .italics()
                                        .color(t.text_sec),
                                );
                            }
                        });
                });
        }
    }

    fn handle_start_clicked(&mut self) {
        self.error_message = None;

        // Apply CPU idle baseline if the user typed one in (empty input = leave
        // the current value alone, e.g. one set by --calibrate-cpu-idle-baseline).
        let trimmed_baseline = self.cpu_idle_baseline_input.trim();
        if !trimmed_baseline.is_empty() {
            match trimmed_baseline.parse::<f64>() {
                Ok(value) if value >= 0.0 && value.is_finite() => {
                    self.monitor.set_cpu_idle_baseline(value);
                }
                _ => {
                    self.error_message = Some(
                        "CPU idle baseline must be a non-negative number in watts.".to_string(),
                    );
                    return;
                }
            }
        }

        // Apply the component filter (CPU only / GPU only / Both) to the
        // CSV / numeric writers and downstream output paths.
        self.outputs.set_component(self.component_filter);

        if self.monitor_mode == MonitorMode::Pid {
            let pid = match self.pid_input.trim().parse::<u32>() {
                Ok(pid) => pid,
                Err(_) => {
                    self.error_message =
                        Some("Invalid PID. Please enter a valid number.".to_string());
                    return;
                }
            };
            if !self.pid_exists(pid) {
                self.error_message = Some(format!(
                    "PID {} not found. Please check and try again.",
                    pid
                ));
                return;
            }
        } else if self.monitor_mode == MonitorMode::App {
            let app_name = self.app_input.trim().to_string();
            if app_name.is_empty() {
                self.error_message =
                    Some("App name is empty. Please enter a valid name.".to_string());
                return;
            }
            if !self.app_exists(&app_name) {
                self.error_message = Some(format!(
                    "Application \"{}\" not found. Please check and try again.",
                    app_name
                ));
                return;
            }
        }

        if self.csv_enabled {
            let Some(path) = &self.csv_path else {
                self.error_message =
                    Some("CSV logging is enabled, but no output file is selected.".to_string());
                return;
            };

            let mut writer = match OutputWriter::new(Some(path), false, self.csv_overwrite) {
                Ok(w) => w,
                Err(e) => {
                    self.error_message =
                        Some(format!("Failed to open CSV file \"{}\": {}", path, e));
                    return;
                }
            };

            let has_process = self.monitor_mode == MonitorMode::Pid;
            let has_app = self.monitor_mode == MonitorMode::App;
            if let Err(e) = writer.write_csv_header(None, has_process, has_app) {
                self.error_message = Some(format!("Failed to write CSV header: {}", e));
                return;
            }

            self.outputs.set_writer(writer);
        }

        // ── Lazy: ring buffer ────────────────────────────────────────────
        // Only attempt to create a writer if the user toggled it on AND we
        // haven't already committed one for this session.
        if self.ringbuffer_enabled && !self.ringbuffer_committed {
            match joularcore::ringbuffer::RingBufferWriter::new(true) {
                Ok(writer) => {
                    self.outputs.set_ringbuffer(Some(writer));
                    self.ringbuffer_committed = true;
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to enable the ring buffer: {}", e));
                    return;
                }
            }
        }

        // ── Lazy: HTTP / WebSocket API server ────────────────────────────
        #[cfg(feature = "api")]
        if self.api_enabled && !self.api_committed {
            let port_str = self.api_port_input.trim();
            if port_str.is_empty() {
                self.error_message = Some(
                    "API is enabled, but no port is specified. Please enter a port between 1 and 65535."
                        .to_string(),
                );
                return;
            }
            let port = match port_str.parse::<u16>() {
                Ok(p) if p > 0 => p,
                _ => {
                    self.error_message = Some(format!(
                        "API port \"{}\" is invalid. Please enter a number between 1 and 65535.",
                        port_str
                    ));
                    return;
                }
            };

            // Validate origins now so we never spawn the server with values
            // tower-http would silently drop later.
            for origin in &self.api_allowed_origins {
                if axum::http::HeaderValue::from_str(origin).is_err() {
                    self.error_message = Some(format!(
                        "Allowed origin \"{}\" is not a valid value.",
                        origin
                    ));
                    return;
                }
            }

            let (tx, _rx) = tokio::sync::broadcast::channel(16);
            match joularcore::common::spawn_api_server(port, self.api_allowed_origins.clone(), tx) {

                Ok((sender, shutdown_tx)) => {
                    self.outputs.set_api_sender(sender);
                    self.api_shutdown_tx = shutdown_tx;
                    self.api_committed = true;
                }
                Err(e) => {
                    self.error_message = Some(format!(
                        "Failed to start the API server on port {}: {}",
                        port, e
                    ));
                    return;
                }
            }
        }

        self.state = ValidationState::Monitor;
        self.reset_sampling_baseline();
        self.monitoring_active = true;
    }
}
