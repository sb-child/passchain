// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use tokio::{time, time::Duration};
use tracing::{info, warn};

pub async fn is_early_stage() -> bool {
    let timeout = time::sleep(Duration::from_secs(3));
    tokio::pin!(timeout);
    let content = tokio::select! {
        x = tokio::fs::read_to_string("/early_cpio") => { x.unwrap_or_default() },
        _ = &mut timeout => {
            warn!("timeout detecting stage, default to false");
            "".to_owned()
        }
    };
    if content.is_empty() {
        // warn!("file not exist or empty");
        return false;
    }
    content.starts_with("1")
}
