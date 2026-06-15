use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::OnceLock;
use std::time::Instant;

use crate::consts::MAX_FPS_REPORT;
use crate::consts::MIN_DELTA_US;
use crate::consts::US_PER_S;

static EPOCH: OnceLock<Instant> = OnceLock::new();
static LAST_US: AtomicU64 = AtomicU64::new(0);

fn compute_fps(dt_us: u64) -> f32 {
    (US_PER_S / dt_us as f32).min(MAX_FPS_REPORT)
}

fn compute_time(now_us: u64) -> f32 {
    (now_us as f32) / US_PER_S
}

pub(crate) fn now_us() -> u64 {
    EPOCH.get_or_init(Instant::now).elapsed().as_micros() as u64
}

pub(crate) fn frame_time_fps() -> (f32, f32) {
    let now = now_us();
    let prev = LAST_US.swap(now, Ordering::Relaxed);
    let dt = now.saturating_sub(prev).max(MIN_DELTA_US);
    (compute_time(now), compute_fps(dt))
}
