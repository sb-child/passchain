use std::{
    fs, io,
    path::Path,
    process::{ExitStatus, Stdio},
};

use tokio::{
    io::{AsyncWriteExt, Interest},
    net::UnixDatagram,
};

static SYSTEMD_REPLY_PASSWORD_BIN: &str = "/lib/systemd/systemd-reply-password";

pub async fn reply_password_by_socket(
    pwd: Option<&str>,
    socket: &Path,
) -> Result<usize, ReplyPasswordSocketError> {
    let socket_str = socket.to_string_lossy().to_string();
    let dg = UnixDatagram::unbound().map_err(ReplyPasswordSocketError::UnixDatagramCreateError)?;
    dg.connect(socket)
        .map_err(|e| ReplyPasswordSocketError::UnixDatagramConnectError(e, socket_str.clone()))?;
    loop {
        let ready = dg.ready(Interest::WRITABLE).await;
        match ready {
            Ok(x) => {
                if x.is_writable() {
                    break;
                } else {
                    return Err(ReplyPasswordSocketError::UnixDatagramNotWritable(
                        socket_str,
                    ));
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
            Err(e) => {
                return Err(ReplyPasswordSocketError::UnixDatagramReadyError(
                    e, socket_str,
                ));
            }
        }
    }
    let r = if let Some(pwd) = pwd {
        dg.send(format!("+{}\0", &pwd).as_bytes()).await
    } else {
        dg.send(format!("-\0").as_bytes()).await
    }
    .map_err(|e| ReplyPasswordSocketError::UnixDatagramWriteError(e, socket_str))?;
    Ok(r)
}

#[derive(thiserror::Error, Debug)]
pub enum ReplyPasswordSocketError {
    #[error("Failed to create unix datagram: {0}")]
    UnixDatagramCreateError(io::Error),
    #[error("Failed to connect to {1} unix datagram: {0}")]
    UnixDatagramConnectError(io::Error, String),
    #[error("Failed to wait for unix datagram {1} ready: {0}")]
    UnixDatagramReadyError(io::Error, String),
    #[error("The unix datagram {0} is not writable")]
    UnixDatagramNotWritable(String),
    #[error("Failed to write to unix datagram {1}: {0}")]
    UnixDatagramWriteError(io::Error, String),
}

pub async fn reply_password_by_systemd_utils(
    pwd: Option<&str>,
    socket: &Path,
) -> Result<(), ReplyPasswordSystemdError> {
    let mut cmd = tokio::process::Command::new(SYSTEMD_REPLY_PASSWORD_BIN);
    if pwd.is_some() {
        cmd.args(&["1", &socket.to_string_lossy()]);
    } else {
        cmd.args(&["0", &socket.to_string_lossy()]);
    }
    let mut proc = cmd
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| ReplyPasswordSystemdError::ProcessSpawnError(e))?;

    if let Some(mut stdin) = proc.stdin.take() {
        if let Some(pwd) = pwd {
            stdin
                .write_all(format!("{pwd}\n").as_bytes())
                .await
                .map_err(|e| ReplyPasswordSystemdError::WriteError(e))?;
        }
    }

    let exit_status = proc
        .wait()
        .await
        .map_err(|e| ReplyPasswordSystemdError::ProcessWaitError(e))?;

    if !exit_status.success() {
        return Err(ReplyPasswordSystemdError::ProcessExitFailure(exit_status));
    }

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum ReplyPasswordSystemdError {
    #[error("Failed to spawn process: {0}")]
    ProcessSpawnError(std::io::Error),

    #[error("Failed to write to stdin: {0}")]
    WriteError(std::io::Error),

    #[error("Failed to wait for process exited: {0}")]
    ProcessWaitError(std::io::Error),

    #[error("Process exited with failure status: {0}")]
    ProcessExitFailure(ExitStatus),
}

pub async fn reply_password(pwd: Option<&str>, socket: &Path) -> Result<(), ReplyPasswordError> {
    let systemd_error = reply_password_by_systemd_utils(pwd, socket).await.err();
    let socket_error = if systemd_error.is_none() {
        reply_password_by_socket(pwd, socket).await.err()
    } else {
        None
    };
    if systemd_error.is_none() && socket_error.is_none() {
        Ok(())
    } else {
        Err(ReplyPasswordError::MethodsError((
            socket_error,
            systemd_error,
        )))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ReplyPasswordError {
    #[error("Methods error: Socket: {}, Systemd: {}", 0.0, 0.1)]
    MethodsError(
        (
            Option<ReplyPasswordSocketError>,
            Option<ReplyPasswordSystemdError>,
        ),
    ),
}
