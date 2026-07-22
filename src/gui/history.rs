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

use std::collections::VecDeque;

#[derive(Clone, Copy, Default)]
pub struct HistoryStats {
    pub min: f64,
    pub avg: f64,
    pub max: f64,
}

pub struct History {
    data: VecDeque<f64>,
    max_len: usize,
}

impl History {
    pub fn new(max_len: usize) -> Self {
        Self {
            data: VecDeque::with_capacity(max_len),
            max_len,
        }
    }

    pub fn push(&mut self, value: f64) {
        if self.data.len() >= self.max_len {
            self.data.pop_front();
        }
        self.data.push_back(value);
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn as_slices(&self) -> (&[f64], &[f64]) {
        self.data.as_slices()
    }

    pub fn iter(&self) -> impl Iterator<Item = f64> + '_ {
        self.data.iter().copied()
    }

    pub fn stats(&self) -> HistoryStats {
        if self.data.is_empty() {
            return HistoryStats::default();
        }

        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        let mut sum = 0.0;

        for value in self.iter() {
            min = min.min(value);
            max = max.max(value);
            sum += value;
        }

        HistoryStats {
            min,
            avg: sum / self.data.len() as f64,
            max,
        }
    }
}
