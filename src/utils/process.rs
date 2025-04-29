// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub fn process_exists(pid: i32) -> bool {
    use nix::errno::Errno;
    use nix::sys::signal::kill;
    use nix::unistd::Pid;
    // kill(pid, 0)
    let r = kill(Pid::from_raw(pid), None);
    match r {
        Ok(_) => false,
        Err(e) => e == Errno::ESRCH,
    }
}
