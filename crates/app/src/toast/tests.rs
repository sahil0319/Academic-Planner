//! R88: unit tests for the toast queue. Time is passed in; no clock, no slint.

use std::time::Duration;

use super::*;

fn secs(s: f32) -> Duration {
    Duration::from_secs_f32(s)
}

fn texts(queue: &ToastQueue, now: Duration) -> Vec<String> {
    queue
        .visible(now)
        .iter()
        .map(|toast| toast.text.clone())
        .collect()
}

fn texts_of(queue: &ToastQueue, now_secs: u64) -> Vec<String> {
    texts(queue, Duration::from_secs(now_secs))
}

/// R88: a repeated text restarts the timer of the toast on screen instead of stacking.
#[test]
fn t84_dedupe_restarts_timer() {
    let mut queue = ToastQueue::new();
    let a = queue.add("A", Duration::from_secs(0));
    queue.add("B", Duration::from_secs(3));
    let a_again = queue.add("A", Duration::from_secs(4));

    assert_eq!(a_again, a, "same text keeps the toast id");
    assert_eq!(
        texts_of(&queue, 4),
        ["A", "B"],
        "one A, one B, A keeps its place"
    );
    assert_eq!(texts_of(&queue, 6), ["A", "B"]);
    assert_eq!(texts_of(&queue, 8), ["A"], "B is over at 3 s + 5 s");
    assert_eq!(
        texts_of(&queue, 9),
        Vec::<String>::new(),
        "A is over at 4 s + 5 s"
    );

    let toast = queue.visible(secs(4.0)).into_iter().next().unwrap();
    assert_eq!(toast.id, a);
    assert!((queue.remaining(toast, secs(4.0)) - 1.0).abs() < 1e-6);
    assert!((queue.remaining(toast, secs(6.5)) - 0.5).abs() < 1e-3);
}

/// R88: the fourth toast removes the oldest; close removes one toast at once.
#[test]
fn t84b_cap_three_and_close() {
    let mut queue = ToastQueue::new();
    let c = queue.add("C", Duration::from_secs(0));
    queue.add("D", Duration::from_secs(1));
    let e = queue.add("E", Duration::from_secs(2));
    queue.add("F", Duration::from_secs(3));

    assert_eq!(
        texts_of(&queue, 3),
        ["D", "E", "F"],
        "oldest dropped, cap three"
    );

    queue.close(e);
    assert_eq!(texts_of(&queue, 3), ["D", "F"]);

    queue.close(e);
    queue.close(c);
    queue.close(9999);
    assert_eq!(
        texts_of(&queue, 3),
        ["D", "F"],
        "unknown or gone id is a no-op"
    );
}

/// R88: `expire` drops exactly the toasts `visible` hides; a toast is gone at start + 5 s.
#[test]
fn t84c_expire_and_visible_agree() {
    let mut queue = ToastQueue::new();
    queue.add("A", Duration::from_secs(0));
    queue.add("B", Duration::from_secs(2));

    for now in [0, 1, 4, 5, 6, 7, 8] {
        let now = Duration::from_secs(now);
        let before = texts(&queue, now);
        queue.expire(now);
        assert_eq!(texts(&queue, now), before, "expire changes nothing visible");
    }
    assert!(queue.visible(Duration::from_secs(8)).is_empty());

    let mut queue = ToastQueue::new();
    queue.add("A", Duration::from_secs(0));
    assert_eq!(texts(&queue, secs(4.999)), ["A"]);
    assert!(
        queue.visible(TOAST_LIFETIME).is_empty(),
        "gone when now >= start + 5 s"
    );
}

/// R88: the timer bar shrinks linearly from 1.0 to 0.0 over 5 s and stays clamped.
#[test]
fn t84d_remaining_is_linear() {
    let mut queue = ToastQueue::new();
    queue.add("A", Duration::from_secs(2));

    let toast = queue
        .visible(Duration::from_secs(2))
        .into_iter()
        .next()
        .unwrap();
    let at = |s: f32| queue.remaining(toast, secs(s));
    assert!((at(2.0) - 1.0).abs() < 1e-6);
    assert!((at(3.0) - 0.8).abs() < 1e-3);
    assert!((at(4.5) - 0.5).abs() < 1e-3);
    assert!((at(6.0) - 0.2).abs() < 1e-3);
    assert!(at(7.0).abs() < 1e-6);
    assert!(at(12.0).abs() < 1e-6, "clamped at empty");
    assert!((at(0.0) - 1.0).abs() < 1e-6, "clamped at full");
}
