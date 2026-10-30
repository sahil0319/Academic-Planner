//! R88: the toast queue. Pure type: no slint and no system clock; the caller passes the time as
//! the `Duration` since app start.
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
    pub fn add(&mut self, text: &str, now: Duration) -> u64 {
        self.expire(now);
        if let Some(toast) = self.toasts.iter_mut().find(|toast| toast.text == text) {
            toast.start = now;
            return toast.id;
        }
        let id = self.next_id;
        self.next_id += 1;
        self.toasts.push(Toast {
            id,
            text: text.to_string(),
            start: now,
        });
        if self.toasts.len() > MAX_TOASTS {
            self.toasts.remove(0);
        }
        id
    }

    /// R88: remove toast `id` at once (unknown id: no-op).
    pub fn close(&mut self, id: u64) {
        self.toasts.retain(|toast| toast.id != id);
    }

    /// R88: drop toasts whose 5 s are over (a toast is gone when `now >= start + 5 s`).
    pub fn expire(&mut self, now: Duration) {
        self.toasts.retain(|toast| is_showing(toast, now));
    }

    /// R88: toasts still showing at `now`, oldest first, newest last.
    pub fn visible(&self, now: Duration) -> Vec<&Toast> {
        self.toasts
            .iter()
            .filter(|toast| is_showing(toast, now))
            .collect()
    }

    /// R88: remaining fraction of the timer bar at `now`, 1.0 full to 0.0 empty, clamped.
    pub fn remaining(&self, toast: &Toast, now: Duration) -> f32 {
        let elapsed = now.saturating_sub(toast.start);
        let left = TOAST_LIFETIME.saturating_sub(elapsed);
        left.as_secs_f32() / TOAST_LIFETIME.as_secs_f32()
    }
}

/// R88: a toast shows while `now < start + 5 s`.
fn is_showing(toast: &Toast, now: Duration) -> bool {
    now < toast.start + TOAST_LIFETIME
}

#[cfg(test)]
mod tests;
