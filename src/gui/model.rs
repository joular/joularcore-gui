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

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ValidationState {
    Options,
    Monitor,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum MonitorMode {
    Cpu,
    Pid,
    App,
}
