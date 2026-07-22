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

use eframe::egui;
use eframe::egui::{Color32, FontData, FontDefinitions, FontFamily, FontId};

const INTER_FONT_NAME: &str = "inter";
const INTER_BYTES: &[u8] = include_bytes!("../../assets/fonts/InterVariable.ttf");



pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
        INTER_FONT_NAME.to_owned(),
        FontData::from_static(INTER_BYTES).into(),
    );

    let mut proportional_family = vec![INTER_FONT_NAME.to_owned()];
    let mut monospace_family = vec![INTER_FONT_NAME.to_owned()];

    proportional_family.extend(
        fonts
            .families
            .get(&FontFamily::Proportional)
            .cloned()
            .unwrap_or_default(),
    );
    monospace_family.extend(
        fonts
            .families
            .get(&FontFamily::Monospace)
            .cloned()
            .unwrap_or_default(),
    );

    fonts
        .families
        .insert(FontFamily::Proportional, proportional_family);
    fonts
        .families
        .insert(FontFamily::Monospace, monospace_family);

    ctx.set_fonts(fonts);
}

#[derive(Clone, Copy)]
pub struct Tokens {
    pub bg: Color32,
    pub surface: Color32,
    pub elevated: Color32,
    pub overlay: Color32,
    pub border: Color32,
    pub titlebar: Color32,
    pub text_pri: Color32,
    pub text_sec: Color32,
    pub text_ter: Color32,

    pub accent: Color32,
    pub cpu: Color32,
    pub gpu: Color32,
    pub usage: Color32,
    pub power: Color32,
    pub success: Color32,
    pub danger: Color32,
}

pub fn tokens(is_dark: bool) -> Tokens {
    // Brand / metric accents — slightly punchier in dark mode and slightly
    // muted in light mode so they sit well against the surrounding surfaces.
    if is_dark {
        let accent = Color32::from_rgb(0x4F, 0x9C, 0xF5);
        let cpu = Color32::from_rgb(0x6E, 0xA9, 0xEC);
        let gpu = Color32::from_rgb(0xEC, 0x7A, 0x7A);
        let usage = Color32::from_rgb(0x6F, 0xC9, 0x97);
        let power = Color32::from_rgb(0xF0, 0xB9, 0x5C);
        let success = usage;
        let danger = gpu;
        Tokens {
            bg: Color32::from_rgb(0x16, 0x18, 0x20),
            surface: Color32::from_rgb(0x1E, 0x21, 0x30),
            elevated: Color32::from_rgb(0x25, 0x28, 0x38),
            overlay: Color32::from_rgb(0x2A, 0x2D, 0x40),
            border: Color32::from_rgb(0x30, 0x33, 0x48),
            titlebar: Color32::from_rgb(0x1A, 0x1C, 0x28),
            text_pri: Color32::from_rgb(0xE8, 0xEA, 0xF2),
            text_sec: Color32::from_rgb(0x7C, 0x80, 0x99),
            text_ter: Color32::from_rgb(0x6E, 0x70, 0x87),
            accent,
            cpu,
            gpu,
            usage,
            power,
            success,
            danger,
        }
    } else {
        let accent = Color32::from_rgb(0x1E, 0x6E, 0xD2);
        let cpu = Color32::from_rgb(0x34, 0x74, 0xC8);
        let gpu = Color32::from_rgb(0xC4, 0x40, 0x40);
        let usage = Color32::from_rgb(0x2D, 0x96, 0x5A);
        let power = Color32::from_rgb(0xA5, 0x77, 0x10);
        let success = usage;
        let danger = gpu;
        Tokens {
            bg: Color32::from_rgb(0xEE, 0xF2, 0xFA),
            surface: Color32::from_rgb(0xFC, 0xFD, 0xFF),
            elevated: Color32::from_rgb(0xE2, 0xE9, 0xF4),
            overlay: Color32::from_rgb(0xD6, 0xE0, 0xEE),
            border: Color32::from_rgb(0xBC, 0xCA, 0xDE),
            titlebar: Color32::from_rgb(0xE8, 0xE9, 0xF4),
            text_pri: Color32::from_rgb(0x18, 0x20, 0x2B),
            text_sec: Color32::from_rgb(0x3C, 0x4C, 0x62),
            text_ter: Color32::from_rgb(0x60, 0x6E, 0x82),
            accent,
            cpu,
            gpu,
            usage,
            power,
            success,
            danger,
        }
    }
}

pub fn apply_theme(ctx: &egui::Context, is_dark_mode: bool) {
    // Fonts and spacing
    let mut style = (*ctx.global_style()).clone();
    style.text_styles = [
        (
            egui::TextStyle::Heading,
            FontId::new(25.0, FontFamily::Proportional),
        ),
        (
            egui::TextStyle::Body,
            FontId::new(14.5, FontFamily::Proportional),
        ),
        (
            egui::TextStyle::Monospace,
            FontId::new(13.5, FontFamily::Monospace),
        ),
        (
            egui::TextStyle::Button,
            FontId::new(14.0, FontFamily::Proportional),
        ),
        (
            egui::TextStyle::Small,
            FontId::new(13.0, FontFamily::Proportional),
        ),
    ]
    .into();
    style.spacing.button_padding = egui::vec2(16.0, 9.0);
    style.spacing.item_spacing = egui::vec2(12.0, 10.0);
    style.spacing.window_margin = egui::Margin::same(18);
    style.spacing.menu_margin = egui::Margin::same(10);
    style.spacing.indent = 18.0;
    ctx.set_global_style(style);

    let t = tokens(is_dark_mode);

    let mut v = if is_dark_mode {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };

    v.panel_fill = t.bg;
    v.window_fill = t.surface;
    v.extreme_bg_color = t.elevated;
    v.faint_bg_color = t.overlay;

    v.override_text_color = Some(t.text_pri);
    v.window_stroke = egui::Stroke::new(1.0_f32, t.border);
    v.window_corner_radius = 16.0.into();
    v.menu_corner_radius = 12.0.into();
    v.popup_shadow = eframe::epaint::Shadow {
        offset: [0, 14],
        blur: 32,
        spread: 0,
        color: Color32::from_black_alpha(if is_dark_mode { 110 } else { 32 }),
    };

    v.widgets.noninteractive.bg_fill = t.elevated;
    v.widgets.noninteractive.weak_bg_fill = t.overlay;
    v.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0_f32, t.border);
    v.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0_f32, t.text_pri);

    v.widgets.inactive.bg_fill = t.elevated;
    v.widgets.inactive.weak_bg_fill = t.overlay;
    v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0_f32, t.border);
    v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0_f32, t.text_pri);

    v.widgets.hovered.bg_fill = t.overlay;
    v.widgets.hovered.weak_bg_fill = t.overlay;
    v.widgets.hovered.bg_stroke = egui::Stroke::new(1.0_f32, t.accent);
    v.widgets.hovered.fg_stroke = egui::Stroke::new(1.5_f32, t.text_pri);

    v.widgets.active.bg_fill = t.accent;
    v.widgets.active.weak_bg_fill = t.accent;
    v.widgets.active.bg_stroke = egui::Stroke::new(1.0_f32, t.accent);
    v.widgets.active.fg_stroke = egui::Stroke::new(1.5_f32, Color32::WHITE);

    v.widgets.open.bg_fill = t.overlay;
    v.widgets.open.weak_bg_fill = t.overlay;
    v.widgets.open.bg_stroke = egui::Stroke::new(1.0_f32, t.border);
    v.widgets.open.fg_stroke = egui::Stroke::new(1.0_f32, t.text_sec);

    v.selection.bg_fill = t.accent;
    v.selection.stroke = egui::Stroke::new(1.0_f32, Color32::WHITE);

    v.hyperlink_color = t.accent;
    v.error_fg_color = t.danger;

    // Unify widget rounding
    v.widgets.inactive.corner_radius = 10.0.into();
    v.widgets.noninteractive.corner_radius = 10.0.into();
    v.widgets.hovered.corner_radius = 10.0.into();
    v.widgets.active.corner_radius = 10.0.into();
    v.widgets.open.corner_radius = 10.0.into();

    ctx.set_visuals(v);
}
