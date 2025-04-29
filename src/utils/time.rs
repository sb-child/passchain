// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

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
