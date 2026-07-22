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

pub mod monitor;
pub mod options;

use crate::gui::theme::tokens;
use crate::gui::{GUI_HEIGHT, GUI_WIDTH, PowerGui};
use eframe::egui;
use eframe::egui::{Color32, RichText};

impl PowerGui {
    pub fn draw_top_bar(&mut self, ui: &mut egui::Ui) {
        let t = tokens(self.is_dark_mode);

        let full_width = ui.available_width();
        let bar_height = 50.0;
        let (bar_rect, _) =
            ui.allocate_exact_size(egui::vec2(full_width, bar_height), egui::Sense::hover());

        // Paint background and border across the full window width, breaking
        // out of the CentralPanel's window_margin by expanding the clip rect.
        let screen = ui.ctx().viewport_rect();
        let full_bar = egui::Rect::from_min_max(
            egui::pos2(screen.left(), screen.top()),
            egui::pos2(screen.right(), bar_rect.bottom()),
        );
        let mut painter = ui.painter().clone();
        painter.set_clip_rect(screen);
        painter.rect_filled(full_bar, 0.0, t.titlebar);
        painter.line_segment(
            [full_bar.left_bottom(), full_bar.right_bottom()],
            egui::Stroke::new(1.0_f32, t.border),
        );

        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(bar_rect.shrink2(egui::vec2(14.0, 9.0)))
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );

        if self.state == crate::gui::model::ValidationState::Monitor {
            if Self::pill_button(&mut child, "Back", t.surface, t.border, t.text_pri).clicked() {
                self.state = crate::gui::model::ValidationState::Options;
                self.outputs.clear_writer();
                self.monitoring_active = false;
                self.reset_sampling_baseline();
                self.cpu_power_history.clear();
                self.gpu_power_history.clear();
                self.total_power_history.clear();
                self.cpu_usage_history.clear();
                self.process_power_history.clear();
                self.app_power_history.clear();

                // Release the ring buffer so it can be re-created on next start.
                if self.ringbuffer_committed {
                    self.outputs.set_ringbuffer(None);
                    self.ringbuffer_committed = false;
                }

                // Shut down the API server so the port is freed for reuse.
                #[cfg(feature = "api")]
                if self.api_committed {
                    if let Some(tx) = self.api_shutdown_tx.take() {
                        let _ = tx.send(());
                    }
                    self.outputs.set_api_sender(None);
                    self.api_committed = false;
                }

                child
                    .ctx()
                    .send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                        GUI_WIDTH, GUI_HEIGHT,
                    )));
            }
        } else {
            draw_brand(&mut child, &t);
        }

        child.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let theme_label = if self.is_dark_mode { "Light" } else { "Dark" };
            if Self::pill_button(ui, theme_label, t.surface, t.border, t.text_sec).clicked() {
                self.is_dark_mode = !self.is_dark_mode;
            }

            if self.state == crate::gui::model::ValidationState::Monitor {
                ui.add_space(6.0);
                let (label, tint) = if self.monitoring_active {
                    ("■ Stop", t.danger)
                } else {
                    ("▶ Resume", t.success)
                };
                let action_fill = tint.gamma_multiply(if self.is_dark_mode { 0.20 } else { 0.14 });
                let action_stroke = tint.gamma_multiply(0.55);

                if Self::pill_button(ui, label, action_fill, action_stroke, tint).clicked() {
                    self.monitoring_active = !self.monitoring_active;
                    if self.monitoring_active {
                        self.reset_sampling_baseline();
                    }
                }
            }
        });

        if self.state == crate::gui::model::ValidationState::Monitor {
            ui.painter().text(
                bar_rect.center(),
                egui::Align2::CENTER_CENTER,
                "Monitoring",
                egui::FontId::proportional(12.5),
                t.text_sec,
            );
        }
    }

    fn pill_button(
        ui: &mut egui::Ui,
        label: &str,
        fill: Color32,
        stroke: Color32,
        text_color: Color32,
    ) -> egui::Response {
        ui.scope(|ui| {
            ui.spacing_mut().button_padding = egui::vec2(12.0, 5.0);

            let response = ui.add(
                egui::Button::new(RichText::new(label).size(11.5).color(text_color))
                    .fill(fill)
                    .stroke(egui::Stroke::new(1.0_f32, stroke))
                    .corner_radius(9.0),
            );

            if response.hovered() || response.has_focus() {
                ui.painter().rect_filled(
                    response.rect,
                    9.0,
                    Color32::WHITE.gamma_multiply(if ui.visuals().dark_mode { 0.06 } else { 0.25 }),
                );
            }

            if response.is_pointer_button_down_on() {
                ui.painter().rect_filled(
                    response.rect,
                    9.0,
                    Color32::BLACK.gamma_multiply(if ui.visuals().dark_mode { 0.15 } else { 0.08 }),
                );
            }

            response
        })
        .inner
    }

    pub fn toggle_ui(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
        let desired_size = egui::vec2(40.0, 22.0);
        let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
        if response.clicked() {
            *on = !*on;
            response.mark_changed();
        }

        response
            .widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, *on, false, ""));

        if ui.is_rect_visible(rect) {
            let how_on = ui.ctx().animate_bool(response.id, *on);
            let visuals = ui.style().interact_selectable(&response, *on);
            let rect = rect.expand(visuals.expansion);
            let radius = 0.5 * rect.height();
            let t = tokens(ui.visuals().dark_mode);

            let bg_color = if *on { t.accent } else { t.text_ter };

            ui.painter().rect(
                rect,
                radius,
                bg_color,
                egui::Stroke::new(1.0_f32, if *on { t.accent } else { t.border }),
                egui::StrokeKind::Inside,
            );

            let circle_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
            let center = egui::pos2(circle_x, rect.center().y);
            ui.painter()
                .circle_filled(center, 0.75 * radius, Color32::WHITE);
        }

        response
    }
}

fn draw_brand(ui: &mut egui::Ui, t: &crate::gui::theme::Tokens) {
    ui.horizontal(|ui| {
        let (icon_rect, _) = ui.allocate_exact_size(egui::vec2(18.0, 18.0), egui::Sense::hover());
        let points = vec![
            egui::pos2(icon_rect.center().x + 1.5, icon_rect.top() + 1.0),
            egui::pos2(icon_rect.left() + 4.5, icon_rect.center().y + 1.0),
            egui::pos2(icon_rect.center().x + 0.5, icon_rect.center().y + 1.0),
            egui::pos2(icon_rect.center().x - 0.5, icon_rect.bottom() - 1.0),
            egui::pos2(icon_rect.right() - 4.0, icon_rect.center().y - 0.5),
            egui::pos2(icon_rect.center().x + 0.5, icon_rect.center().y - 0.5),
        ];
        ui.painter().add(egui::Shape::convex_polygon(
            points,
            t.accent,
            egui::Stroke::NONE,
        ));
        ui.add_space(6.0);
        ui.label(
            RichText::new("Joular Core")
                .size(18.0)
                .strong()
                .color(t.text_pri),
        );
    });
}
