//! R88: the toast queue. Pure type: no slint and no system clock; the caller passes the time as
//! the `Duration` since app start.
//!
//! M18 stub: bodies are `todo!()` until the builder implements them.
#![allow(dead_code)]

use std::time::Duration;

/// R88: how long a toast shows before its timer bar is empty.
pub const TOAST_LIFETIME: Duration = Duration::from_secs(5);

/// R88: the most toasts on screen at once.
pub const MAX_TOASTS: usize = 3;

/// R88: one toast on screen.
#[derive(Debug, Clone, PartialEq)]
pub struct Toast {
    pub id: u64,
    pub text: String,
    start: Duration,
}

/// R88: the toasts on screen, oldest first.
#[derive(Debug, Default)]
pub struct ToastQueue {
    toasts: Vec<Toast>,
    next_id: u64,
}

impl ToastQueue {
    pub fn new() -> Self {
        Self::default()
    }

    /// R88: add; same text on screen restarts that toast's timer, keeps its position and id. A
    /// fourth toast removes the oldest. Returns the toast id.
    pub fn add(&mut self, _text: &str, _now: Duration) -> u64 {
        todo!("R88: add with dedupe and cap")
    }

    /// R88: remove toast `id` at once (unknown id: no-op).
    pub fn close(&mut self, _id: u64) {
        todo!("R88: close")
    }

    /// R88: drop toasts whose 5 s are over (a toast is gone when `now >= start + 5 s`).
    pub fn expire(&mut self, _now: Duration) {
        todo!("R88: expire")
    }

    /// R88: toasts still showing at `now`, oldest first, newest last.
    pub fn visible(&self, _now: Duration) -> Vec<&Toast> {
        todo!("R88: visible")
    }

    /// R88: remaining fraction of the timer bar at `now`, 1.0 full to 0.0 empty, clamped.
    pub fn remaining(&self, _toast: &Toast, _now: Duration) -> f32 {
        todo!("R88: remaining")
    }
}

#[cfg(test)]
mod tests;
