pub fn get_clock_monotonic() -> Option<i64> {
    use nix::time::{ClockId, clock_gettime};
    let tm = clock_gettime(ClockId::CLOCK_MONOTONIC);
    match tm {
        Ok(t) => Some(t.tv_nsec() as i64),
        Err(e) => {
            tracing::error!("get_clock_monotonic: clock_gettime() error: {}", e);
            None
        }
    }
}
