// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::path::{Path, PathBuf};

use notify::{RecursiveMode, Watcher};
use tokio::{io::AsyncReadExt, sync};

use crate::utils::{
    process::process_exists,
    systemd::{ReplyPasswordError, reply_password},
    time::get_clock_monotonic,
};

#[derive(Debug)]
pub struct AgentFile {
    file: tokio::fs::File,
    path: PathBuf,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct AgentRequest {
    #[serde(rename = "Ask")]
    pub ask: AgentRequestAskField,
    #[serde(skip)]
    pub filepath: PathBuf,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct AgentRequestAskField {
    #[serde(rename = "Message")]
    pub message: Option<String>,
    #[serde(rename = "Icon")]
    pub icon: Option<String>,
    #[serde(rename = "PID")]
    pub pid: i32,
    #[serde(rename = "Echo")]
    pub echo: Option<i32>,
    #[serde(rename = "Socket")]
    pub socket: PathBuf,
    /// CLOCK_MONOTONIC in usecs
    #[serde(rename = "NotAfter")]
    pub not_after: i64,
}

pub struct AgentListener {
    request_chan: sync::mpsc::Receiver<AgentRequest>,
    request_task: tokio::task::JoinHandle<()>,
    notify_watcher: Box<dyn notify::Watcher>,
    ask_password_path: PathBuf,
}

impl AgentListener {
    pub fn new() -> Result<Self, ListenError> {
        let ask_password_path = Path::new("/run/systemd/ask-password");
        let (file_chan_tx, mut file_chan_rx) = sync::mpsc::channel(16);
        let (request_chan_tx, request_chan_rx) = sync::mpsc::channel(16);
        let watcher = notify::recommended_watcher(move |ev: notify::Result<notify::Event>| {
            let ev = match ev {
                Ok(x) => x,
                Err(e) => {
                    tracing::error!("notify watcher: event error: {e}");
                    return;
                }
            };
            // IN_CLOSE_WRITE | IN_MOVED_TO
            if !(ev.kind
                == notify::EventKind::Access(notify::event::AccessKind::Close(
                    notify::event::AccessMode::Write,
                ))
                || ev.kind
                    == notify::EventKind::Modify(notify::event::ModifyKind::Name(
                        notify::event::RenameMode::To,
                    )))
            {
                tracing::info!("notify watcher: incorrect kind, ignoring: {ev:?}");
                return;
            }
            let file_path = if let Some(x) = ev.paths.last() {
                x
            } else {
                tracing::info!("notify watcher: failed to get file path, ignoring: {ev:?}");
                return;
            };
            let file_name = if let Some(x) = file_path.file_name() {
                x
            } else {
                tracing::info!("notify watcher: failed to get file name, ignoring: {ev:?}");
                return;
            };
            if !file_name.to_string_lossy().to_string().starts_with("ask.") {
                tracing::info!(
                    "notify watcher: file name is not starts with \"ask.\", ignoring: {ev:?}"
                );
                return;
            }
            let file_chan_tx = file_chan_tx.clone();
            let file_path = file_path.clone();
            tokio::task::spawn(async move {
                let file = match tokio::fs::File::open(&file_path).await {
                    Ok(f) => f,
                    Err(e) => {
                        tracing::error!("notify watcher: task: failed to open {file_path:?}: {e}");
                        return;
                    }
                };
                if let Err(e) = file_chan_tx
                    .send(AgentFile {
                        file,
                        path: file_path.clone(),
                    })
                    .await
                {
                    tracing::error!("notify watcher: task: failed to send {file_path:?}: {e}");
                    return;
                }
                tracing::info!("notify watcher: task: done for {file_path:?}");
            });
        });
        let req_task = tokio::task::spawn(async move {
            while let Some(mut file) = file_chan_rx.recv().await {
                let request_chan_tx = request_chan_tx.clone();
                tokio::spawn(async move {
                    tracing::info!("request task: processing {file:?}");
                    let metadata = file.file.metadata().await;
                    let metadata = match metadata {
                        Ok(x) => x,
                        Err(e) => {
                            tracing::error!("request task: failed to read metadata {file:?}: {e}");
                            return;
                        }
                    };
                    let file_size = metadata.len();
                    let mut file_buf = Vec::with_capacity(file_size as usize);
                    match file.file.read_buf(&mut file_buf).await {
                        Ok(x) => {
                            tracing::info!("request task: read {x} bytes {file:?}");
                        }
                        Err(e) => {
                            tracing::error!("request task: failed to read {file:?}: {e}");
                            return;
                        }
                    };
                    let file_str = match String::from_utf8(file_buf) {
                        Ok(x) => x,
                        Err(e) => {
                            tracing::error!("request task: failed to decode utf8 {file:?}: {e}");
                            return;
                        }
                    };
                    let file_path = file.path.clone();
                    let file_path_2 = file.path;
                    let req = tokio::task::spawn_blocking(move || {
                        let cfg = serde_ini::from_str::<AgentRequest>(&file_str);
                        match cfg {
                            Ok(x) => Some(x),
                            Err(e) => {
                                tracing::error!(
                                    "request task: failed to deserialize {}: {e}",
                                    file_path.to_string_lossy()
                                );
                                return None;
                            }
                        }
                    })
                    .await;
                    let req = match req {
                        Ok(x) => x,
                        Err(e) => {
                            tracing::error!(
                                "request task: failed to deserialize {}: {e}",
                                file_path_2.to_string_lossy()
                            );
                            return;
                        }
                    };
                    if let Some(x) = req {
                        match request_chan_tx.send(x).await {
                            Ok(_) => {
                                tracing::info!(
                                    "request task: done for {}",
                                    file_path_2.to_string_lossy()
                                );
                            }
                            Err(e) => {
                                tracing::error!(
                                    "request task: failed to send {}: {e}",
                                    file_path_2.to_string_lossy()
                                );
                            }
                        };
                    } else {
                        tracing::error!(
                            "request task: failed to processing {}, see errors above",
                            file_path_2.to_string_lossy()
                        );
                    }
                });
            }
        });
        let mut watcher = match watcher {
            Ok(x) => x,
            Err(e) => {
                tracing::error!("failed to create notify watcher: {e}");
                return Err(ListenError::NotifyWatcherError(e));
            }
        };
        if let Err(e) = watcher.watch(ask_password_path, RecursiveMode::NonRecursive) {
            tracing::error!(
                "failed to watch {}: {e}",
                ask_password_path.to_string_lossy()
            );
            return Err(ListenError::NotifyWatcherError(e));
        }
        Ok(Self {
            request_chan: request_chan_rx,
            request_task: req_task,
            notify_watcher: Box::new(watcher),
            ask_password_path: ask_password_path.to_path_buf(),
        })
    }

    pub async fn next_task(&mut self) -> Option<AgentTask> {
        let task = self.request_chan.recv().await?;
        Some(AgentTask {
            filepath: task.filepath,
            pid: task.ask.pid,
            message: task.ask.message,
            socket: task.ask.socket,
            not_after: task.ask.not_after,
        })
    }
}

impl Drop for AgentListener {
    fn drop(&mut self) {
        self.request_chan.close();
        self.request_task.abort();
        self.notify_watcher.unwatch(&self.ask_password_path).ok();
    }
}

pub struct AgentTask {
    filepath: PathBuf,
    pid: i32,
    message: Option<String>,
    socket: PathBuf,
    not_after: i64,
}

impl AgentTask {
    pub fn closed(&self) -> bool {
        let now = get_clock_monotonic();
        if let Some(now) = now {
            if self.not_after > 0 {
                if now > self.not_after {
                    return true;
                }
            }
        }
        if !process_exists(self.pid) {
            return true;
        }
        match std::fs::exists(self.filepath.clone()) {
            Ok(x) => {
                if !x {
                    return true;
                }
            }
            Err(e) => {
                tracing::error!("Failed to check {} exists: {e}", self.filepath.display())
            }
        }
        false
    }

    pub fn message(&self) -> Option<String> {
        self.message.clone()
    }

    pub async fn answer(&self, payload: Option<String>) -> Result<(), AnswerError> {
        if self.closed() {
            return Err(AnswerError::Closed);
        }
        reply_password(payload.as_deref(), &self.socket)
            .await
            .map_err(AnswerError::ReplyError)?;
        Ok(())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ListenError {
    #[error("Notify watcher error: {0}")]
    NotifyWatcherError(notify::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum AnswerError {
    #[error("This agent was closed")]
    Closed,

    #[error("Failed to reply password: {0}")]
    ReplyError(ReplyPasswordError),
}
