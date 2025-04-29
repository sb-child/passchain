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
