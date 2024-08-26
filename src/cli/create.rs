// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::sync::{
    atomic::{AtomicBool, AtomicU64},
    Arc,
};

use crate::errors;
use base64::Engine;
use clap::{Parser, Subcommand};

use indicatif::{ProgressBar, ProgressIterator};
use inquire::InquireError;

use strum::{Display, EnumDiscriminants, EnumIter, EnumString, IntoEnumIterator};
use tracing::{info_span, Span};
use tracing_indicatif::span_ext::IndicatifSpanExt;

const BLOCK_SIZE: usize = 128;
type Block = [u8; BLOCK_SIZE];

type BlockSender = tokio::sync::oneshot::Sender<Block>;
type BlockReceiver = tokio::sync::oneshot::Receiver<Block>;

type StringSender = tokio::sync::oneshot::Sender<String>;
type StringReceiver = tokio::sync::oneshot::Receiver<String>;

type FidoSender = tokio::sync::oneshot::Sender<FidoItem>;
type FidoReceiver = tokio::sync::oneshot::Receiver<FidoItem>;

type TaskErrorSender = tokio::sync::oneshot::Sender<Option<errors::TaskError>>;
type TaskErrorReceiver = tokio::sync::oneshot::Receiver<Option<errors::TaskError>>;

type AskPinContent = (String, StringSender);

type AskPinSender = tokio::sync::mpsc::Sender<AskPinContent>;
type AskPinReceiver = tokio::sync::mpsc::Receiver<AskPinContent>;

fn new_block_channel() -> (BlockSender, BlockReceiver) {
    tokio::sync::oneshot::channel()
}

fn new_string_channel() -> (StringSender, StringReceiver) {
    tokio::sync::oneshot::channel()
}

fn new_fido_channel() -> (FidoSender, FidoReceiver) {
    tokio::sync::oneshot::channel()
}

fn new_task_error_channel() -> (TaskErrorSender, TaskErrorReceiver) {
    tokio::sync::oneshot::channel()
}

fn new_ask_pin_channel() -> (AskPinSender, AskPinReceiver) {
    tokio::sync::mpsc::channel(1)
}

fn new_hasher<'k>() -> argon2::Argon2<'k> {
    argon2::Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(1000000, 15, 1, Some(BLOCK_SIZE)).unwrap(),
        // argon2::Params::new(1000000, 30, 1, Some(BLOCK_SIZE)).unwrap(),
    )
}

fn new_random_block() -> Block {
    use ring::rand::{SecureRandom, SystemRandom};
    let rnd = SystemRandom::new();
    let mut tmp: Block = [0; BLOCK_SIZE];
    rnd.fill(&mut tmp).unwrap();
    tmp
}

fn new_block() -> Block {
    [0; BLOCK_SIZE]
}

#[derive(Parser, Debug)]
#[command()]
pub struct Args {}

impl Args {
    pub fn build(self) -> anyhow::Result<Executor, errors::PasschainError> {
        Ok(Executor {})
    }
}

pub struct Executor {}

impl Executor {
    pub async fn execute(mut self) -> anyhow::Result<(), errors::PasschainError> {
        let prompt_thread = tokio::task::spawn_blocking(|| prompt_factors());
        let factors = prompt_thread.await??;
        let (hash, pre, post) = self.compute(factors).await?;
        Ok(())
    }

    async fn compute(
        &mut self,
        factors: Vec<Factor>,
    ) -> anyhow::Result<(Block, Block, Block), errors::PasschainError> {
        // pre_bytes ->    any_factor(0)  -> any_factor(1..) -> post_bytes
        //      |              |                    |               |
        //      V              V                    V               V
        //    hash   <-       hash        <-      hash       <-   block
        //      |
        //      v
        //   result

        tracing::info!("Computing hash, please wait...");

        // progress
        let prog_all = Arc::new(AtomicU64::new(0));
        let prog_completed = Arc::new(AtomicU64::new(0));
        let done = Arc::new(AtomicBool::new(false));
        async fn prog_fn<F: std::future::Future>(
            x: F,
            pa: Arc<AtomicU64>,
            pc: Arc<AtomicU64>,
        ) -> tokio::task::JoinHandle<<F as std::future::Future>::Output>
        where
            <F as std::future::Future>::Output: Send,
            <F as std::future::Future>::Output: 'static,
        {
            tokio::spawn({
                pa.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let x = x.await;
                pc.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                async { x }
            })
        }

        // status and askpin
        // todo
        let (askpin_tx, mut askpin_rx) = new_ask_pin_channel();
        // let curr_all = prog_all.clone();
        // let curr_completed = prog_completed.clone();
        let curr_done = done.clone();
        // let pb = ProgressBar::new(1);
        tokio::task::spawn_blocking(move || {
            // todo
            loop {
                // let pb = info_span!("Compute progress");
                // pb.pb_set_style(&indicatif::ProgressStyle::default_bar());
                // let pbe = pb.enter();
                // let curr_all = curr_all.load(std::sync::atomic::Ordering::Relaxed);
                // let curr_completed = curr_completed.load(std::sync::atomic::Ordering::Relaxed);
                let curr_done = curr_done.load(std::sync::atomic::Ordering::Relaxed);
                // if curr_all > 0 {
                //     pb.pb_start();
                //     pb.pb_set_length(curr_all);
                //     pb.pb_set_position(curr_completed);
                // }
                match askpin_rx.try_recv() {
                    Ok(askpin) => {
                        // drop(pbe);
                        // drop(pb);
                        let ans = inquire::Password::new(&askpin.0)
                            .with_display_mode(inquire::PasswordDisplayMode::Masked)
                            .with_display_toggle_enabled()
                            .with_help_message("press Enter to finish, press ESC or Ctrl-C to cancel, press Ctrl-R to toggle.")
                            .prompt();
                        match ans {
                            Ok(pwd) => {
                                askpin.1.send(pwd).unwrap();
                            }
                            Err(e) => {
                                tracing::error!("Inquire error during askpin: {:?}", e);
                                drop(askpin.1);
                            }
                        }
                    }
                    Err(_e) => std::thread::sleep(tokio::time::Duration::from_millis(1)),
                };
                if curr_done {
                    break;
                }
            }
        });

        // nonce
        let (pre_nonce_tx, pre_nonce_rx) = new_block_channel();
        let (post_nonce_tx, post_nonce_rx) = new_block_channel();

        // input args
        let (pre_input_tx, pre_input_rx) = new_block_channel();
        let (post_input_tx, post_input_rx) = new_block_channel();

        // output args
        let (pre_tx, pre_rx) = new_block_channel();
        let (post_tx, post_rx) = new_block_channel();

        let factor_num = factors.len();

        // factor layer
        let mut last_rx = Some(pre_input_rx);
        let mut copied_rx = vec![];
        let mut factor_tasks = vec![];
        let mut copier_tasks = vec![];
        for f in factors.iter() {
            let (res_tx, res_rx) = new_block_channel();
            let (cp1_tx, cp1_rx) = new_block_channel();
            let (cp2_tx, cp2_rx) = new_block_channel();
            let cp = tokio::spawn(
                Task::Copier {
                    input: last_rx.replace(res_rx).unwrap(),
                    output1: cp1_tx,
                    output2: cp2_tx,
                }
                .run(),
            );
            copier_tasks.push(prog_fn(cp, prog_all.clone(), prog_completed.clone()));
            copied_rx.push(cp2_rx);
            let task = match f {
                Factor::Password(x) => {
                    let (pwd_tx, pwd_rx) = new_string_channel();
                    pwd_tx.send(x.clone()).unwrap();
                    tokio::spawn(
                        Task::PasswordFactor {
                            pwd: pwd_rx,
                            prev: cp1_rx,
                            res: res_tx,
                        }
                        .run(),
                    )
                }
                Factor::Fido(x) => {
                    let (dev_tx, dev_rx) = new_fido_channel();
                    dev_tx.send(x.clone()).unwrap();
                    tokio::spawn(
                        Task::FidoFactor {
                            pwd: askpin_tx.clone(),
                            dev: dev_rx,
                            prev: cp1_rx,
                            res: res_tx,
                        }
                        .run(),
                    )
                }
            };
            factor_tasks.push(prog_fn(task, prog_all.clone(), prog_completed.clone()));
        }

        let (factor_hash_tx, factor_hash_rx) = new_block_channel();
        let factor_layer_hasher = prog_fn(
            tokio::spawn(
                Task::Hasher {
                    pwd: post_input_rx,
                    salt: last_rx.take().unwrap(),
                    res: factor_hash_tx,
                }
                .run(),
            ),
            prog_all.clone(),
            prog_completed.clone(),
        );

        // hash layer
        let mut last_rx = Some(factor_hash_rx);
        let mut hasher_tasks = vec![];
        for i in (0..factor_num).rev() {
            let (res_tx, res_rx) = new_block_channel();
            hasher_tasks.push(prog_fn(
                tokio::spawn(
                    Task::Hasher {
                        pwd: copied_rx.remove(i),
                        salt: last_rx.replace(res_rx).unwrap(),
                        res: res_tx,
                    }
                    .run(),
                ),
                prog_all.clone(),
                prog_completed.clone(),
            ));
        }

        // copier
        let pre_copier = prog_fn(
            tokio::spawn(
                Task::Copier {
                    input: pre_nonce_rx,
                    output1: pre_input_tx,
                    output2: pre_tx,
                }
                .run(),
            ),
            prog_all.clone(),
            prog_completed.clone(),
        );
        let post_copier = prog_fn(
            tokio::spawn(
                Task::Copier {
                    input: post_nonce_rx,
                    output1: post_input_tx,
                    output2: post_tx,
                }
                .run(),
            ),
            prog_all.clone(),
            prog_completed.clone(),
        );

        // generate initial params
        let pre_generator = prog_fn(
            tokio::spawn(
                Task::Nonce {
                    nonce: pre_nonce_tx,
                }
                .run(),
            ),
            prog_all.clone(),
            prog_completed.clone(),
        );
        let post_generator = prog_fn(
            tokio::spawn(
                Task::Nonce {
                    nonce: post_nonce_tx,
                }
                .run(),
            ),
            prog_all.clone(),
            prog_completed.clone(),
        );

        let mut all_tasks = vec![
            post_generator,
            pre_generator,
            post_copier,
            pre_copier,
            factor_layer_hasher,
        ];
        all_tasks.append(&mut factor_tasks);
        all_tasks.append(&mut copier_tasks);
        all_tasks.append(&mut hasher_tasks);

        let all_tasks = futures::future::join_all(futures::future::join_all(all_tasks).await).await;

        for t in all_tasks {
            let t = t.unwrap().unwrap();
            match t.await {
                Ok(x) => match x {
                    Some(te) => {
                        done.fetch_update(
                            std::sync::atomic::Ordering::Relaxed,
                            std::sync::atomic::Ordering::Relaxed,
                            |_| Some(true),
                        )
                        .unwrap();
                        tracing::error!("Task error: {:?}", te);
                    }
                    None => {}
                },
                Err(e) => {
                    tracing::error!("Receive error: {:?}", e);
                    done.fetch_update(
                        std::sync::atomic::Ordering::Relaxed,
                        std::sync::atomic::Ordering::Relaxed,
                        |_| Some(true),
                    )
                    .unwrap();
                }
            }
        }

        let result_rx = last_rx.take().unwrap().await.unwrap();
        let pre = pre_rx.await.unwrap();
        let post = post_rx.await.unwrap();
        done.fetch_update(
            std::sync::atomic::Ordering::Relaxed,
            std::sync::atomic::Ordering::Relaxed,
            |_| Some(true),
        )
        .unwrap();

        tracing::info!(
            "Hash computed successfully, hash={}, pre={}, post={}",
            hex::encode(result_rx),
            hex::encode(pre),
            hex::encode(post),
        );

        Ok((result_rx, pre, post))
    }
}

fn prompt_factors() -> anyhow::Result<Vec<Factor>, errors::PasschainError> {
    let mut factors = vec![];
    'next_factor: for i in 1..u32::MAX {
        'retry: loop {
            let fa = match FactorDiscriminants::ask(i) {
                Ok(f) => Factor::ask(f),
                Err(e) => match e {
                    e @ errors::AskError::InquireError(_) => {
                        return Err(errors::PasschainError::AskError(e))
                    }
                    errors::AskError::Interrupted => {
                        return Err(errors::PasschainError::AskError(e))
                    }
                    errors::AskError::Canceled => break 'next_factor,
                },
            };
            let fa = match fa {
                Ok(f) => f,
                Err(e) => match e {
                    errors::AskError::InquireError(e) => match e {
                        InquireError::InvalidConfiguration(_) => {
                            tracing::error!("No option or device available at this time.");
                            continue 'retry;
                        }
                        oe @ _ => {
                            return Err(errors::PasschainError::AskError(
                                errors::AskError::InquireError(oe),
                            ))
                        }
                    },
                    errors::AskError::Interrupted => {
                        return Err(errors::PasschainError::AskError(e))
                    }
                    errors::AskError::Canceled => continue 'retry,
                },
            };
            let fac = fa.clone();
            let fac_name = FactorDiscriminants::from(fa.clone());
            let fido_fa = match fa {
                Factor::Fido(f) => f,
                _ => {
                    tracing::info!("Factor {i} with {fac_name} added.");
                    factors.push(fac);
                    continue 'next_factor;
                }
            };
            match fido_fa.ask() {
                Ok(x) => {
                    if !x {
                        continue 'retry;
                    }
                }
                Err(e) => match e {
                    e @ errors::AskError::InquireError(_) => {
                        return Err(errors::PasschainError::AskError(e))
                    }
                    errors::AskError::Interrupted => {
                        return Err(errors::PasschainError::AskError(e))
                    }
                    errors::AskError::Canceled => continue 'retry,
                },
            }
            tracing::info!("Factor {i} with {fac_name} added.");
            factors.push(fac);
            continue 'next_factor;
        }
    }
    Ok(factors)
}

#[derive(Clone)]
struct FidoItem {
    info: ctap_hid_fido2::HidInfo,
    device_name: String,
}

impl std::fmt::Display for FidoItem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.device_name)
    }
}

impl std::fmt::Debug for FidoItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FidoItem")
            .field("device_name", &self.device_name)
            .finish()
    }
}

impl FidoItem {
    fn ask(&self) -> Result<bool, errors::AskError> {
        let title = if self.trigger_wink() {
            "Wink command sent, is this the exact device you selected?"
        } else {
            "This device is not responding to wink command, continue anyway?"
        };
        let ans = inquire::Confirm::new(title)
            .with_default(false)
            .with_help_message("Type y to accept, type n or press ESC to cancel.")
            .prompt();
        match ans {
            Ok(choice) => Ok(choice),
            Err(e) => Err(e.into()),
        }
    }

    fn open(&self) -> anyhow::Result<ctap_hid_fido2::FidoKeyHid> {
        let current_param = self.info.param.clone();
        let dev = ctap_hid_fido2::FidoKeyHidFactory::create_by_params(
            &[current_param],
            &ctap_hid_fido2::Cfg::init(),
        );
        dev
    }

    fn trigger_wink(&self) -> bool {
        let pb = ProgressBar::new(2);
        let dev = self.open();
        let fido = match dev {
            Ok(x) => x,
            Err(e) => {
                tracing::error!("Unable to open the device {0}: {e}", self.device_name);
                pb.finish_and_clear();
                return false;
            }
        };
        pb.inc(1);
        let wk = fido.wink();
        pb.finish_and_clear();
        match wk {
            Ok(_) => true,
            Err(e) => {
                tracing::error!("Failed to wink {0}: {e}", self.device_name);
                false
            }
        }
    }
}

fn get_fido_list() -> Vec<FidoItem> {
    let devs = ctap_hid_fido2::get_fidokey_devices();
    let mut r = vec![];
    for info in devs.into_iter().progress() {
        let current_info = info.clone();
        let current_param = info.param.clone();
        let dev = ctap_hid_fido2::FidoKeyHidFactory::create_by_params(
            &[info.param],
            &ctap_hid_fido2::Cfg::init(),
        );
        let device_name = match current_param {
            ctap_hid_fido2::HidParam::VidPid { vid, pid } => {
                format!("{:04x}:{:04x}", vid, pid)
            }
            ctap_hid_fido2::HidParam::Path(x) => {
                format!("{:04x}:{:04x} ({})", info.vid, info.pid, x)
            }
        };
        match dev {
            Ok(_x) => {
                let n = format!("{} | {}", info.product_string, device_name);
                r.push(FidoItem {
                    info: current_info,
                    device_name: n,
                });
            }
            Err(e) => {
                tracing::error!("Unable to open the device {device_name}: {e}");
            }
        }
    }
    r
}

#[derive(EnumDiscriminants, Clone)]
#[strum_discriminants(derive(EnumIter, EnumString, Display))]
enum Factor {
    Password(String),
    Fido(FidoItem),
}

impl Factor {
    fn ask(selected: FactorDiscriminants) -> Result<Self, errors::AskError> {
        match selected {
            FactorDiscriminants::Password => {
                let ans = inquire::Password::new("Input a password for this factor:")
                    .with_display_mode(inquire::PasswordDisplayMode::Masked)
                    .with_display_toggle_enabled()
                    .with_help_message("press Enter to finish, press ESC to go back, press Ctrl-C to exit, press Ctrl-R to toggle.")
                    .prompt();
                match ans {
                    Ok(choice) => Ok(Self::Password(choice)),
                    Err(e) => Err(e.into()),
                }
            }
            FactorDiscriminants::Fido => {
                let list = get_fido_list();
                let ans: Result<FidoItem, InquireError> =
            inquire::Select::new(&format!("Select a FIDO key to store this factor:"), list)
                .with_help_message("Do not remove the key until the process is complete. Press up/down to navigate, press Enter to select, press ESC to go back, press Ctrl-C to exit.")
                .prompt();
                match ans {
                    Ok(choice) => Ok(Self::Fido(choice)),
                    Err(e) => Err(e.into()),
                }
            }
        }
    }
}

impl FactorDiscriminants {
    fn ask(n: u32) -> Result<Self, errors::AskError> {
        let options: Vec<_> = FactorDiscriminants::iter().collect();
        let ans: Result<FactorDiscriminants, InquireError> =
            inquire::Select::new(&format!("[{n}] Add a factor:"), options)
                .with_help_message("Be sure to remember the order of the factors, otherwise you will not be able to decrypt it later. Press up/down to navigate, press Enter to select, press ESC to finish, press Ctrl-C to exit.")
                .prompt();
        match ans {
            Ok(choice) => Ok(choice),
            Err(e) => Err(e.into()),
        }
    }
}

enum Task {
    Nonce {
        nonce: BlockSender,
    },
    Copier {
        input: BlockReceiver,
        output1: BlockSender,
        output2: BlockSender,
    },
    Hasher {
        pwd: BlockReceiver,
        salt: BlockReceiver,
        res: BlockSender,
    },
    PasswordFactor {
        pwd: StringReceiver,
        prev: BlockReceiver,
        res: BlockSender,
    },
    FidoFactor {
        pwd: AskPinSender,
        dev: FidoReceiver,
        prev: BlockReceiver,
        res: BlockSender,
    },
}

impl Task {
    async fn run(self) -> TaskErrorReceiver {
        let (err_tx, err_rx) = new_task_error_channel();
        match self {
            Task::Nonce { nonce } => {
                err_tx.send(nonce_task(nonce).await.err()).unwrap();
            }
            Task::Copier {
                input,
                output1,
                output2,
            } => {
                err_tx
                    .send(copier_task(input, output1, output2).await.err())
                    .unwrap();
            }
            Task::Hasher { pwd, salt, res } => {
                err_tx
                    .send(hasher_task(pwd, salt, res).await.err())
                    .unwrap();
            }
            Task::PasswordFactor { pwd, prev, res } => {
                err_tx
                    .send(password_factor_task(pwd, prev, res).await.err())
                    .unwrap();
            }
            Task::FidoFactor {
                pwd,
                dev,
                prev,
                res,
            } => {
                err_tx
                    .send(fido_factor_task(pwd, dev, prev, res).await.err())
                    .unwrap();
            }
        };
        err_rx
    }
}

async fn nonce_task(nonce: BlockSender) -> anyhow::Result<(), errors::TaskError> {
    let x = tokio::task::spawn_blocking(|| new_random_block()).await?;
    if let Err(_) = nonce.send(x) {
        Err(errors::TaskError::ReceiverDropped)
    } else {
        Ok(())
    }
}

async fn copier_task(
    input: BlockReceiver,
    output1: BlockSender,
    output2: BlockSender,
) -> anyhow::Result<(), errors::TaskError> {
    let input = match input.await {
        Ok(x) => x,
        Err(_) => return Err(errors::TaskError::SenderDropped),
    };
    async fn cp(x: Block, y: BlockSender) {
        if let Err(_) = y.send(x) {}
    }
    tokio::join!(cp(input.clone(), output1), cp(input, output2));
    Ok(())
}

async fn hasher_task(
    pwd: BlockReceiver,
    salt: BlockReceiver,
    res: BlockSender,
) -> anyhow::Result<(), errors::TaskError> {
    let pwd = match pwd.await {
        Ok(x) => x,
        Err(_) => return Err(errors::TaskError::SenderDropped),
    };
    let salt = match salt.await {
        Ok(x) => x,
        Err(_) => return Err(errors::TaskError::SenderDropped),
    };
    let x = tokio::task::spawn_blocking(move || {
        let argon = new_hasher();
        let mut out = new_block();
        argon.hash_password_into(&pwd, &salt, &mut out).unwrap();
        out
    })
    .await?;
    if let Err(_) = res.send(x) {
        Err(errors::TaskError::ReceiverDropped)
    } else {
        tracing::debug!("hasher_task done.");
        Ok(())
    }
}

async fn password_factor_task(
    pwd: StringReceiver,
    prev: BlockReceiver,
    res: BlockSender,
) -> anyhow::Result<(), errors::TaskError> {
    let hasher = new_hasher();
    let pwd = match pwd.await {
        Ok(x) => x,
        Err(_) => return Err(errors::TaskError::SenderDropped),
    };
    let prev = match prev.await {
        Ok(x) => x,
        Err(_) => return Err(errors::TaskError::SenderDropped),
    };
    let out = tokio::task::spawn_blocking(move || {
        let pwd = pwd.as_bytes();
        let mut out = new_block();
        match hasher.hash_password_into(&pwd, &prev, &mut out) {
            Ok(_) => Ok(out),
            Err(e) => Err(errors::TaskError::HasherError(e)),
        }
    })
    .await??;
    tracing::debug!("password_factor_task done.");
    res.send(out).unwrap();
    Ok(())
}

async fn fido_factor_task(
    pwd: AskPinSender,
    dev: FidoReceiver,
    prev: BlockReceiver,
    res: BlockSender,
) -> anyhow::Result<(), errors::TaskError> {
    use base64::engine::general_purpose::URL_SAFE;
    use ctap_hid_fido2::{
        fidokey::{
            AssertionExtension, CredentialExtension, GetAssertionArgsBuilder,
            MakeCredentialArgsBuilder,
        },
        verifier::create_challenge,
    };
    use std::sync::Arc;
    let chall = tokio::task::spawn_blocking(|| create_challenge());
    let chall_2 = tokio::task::spawn_blocking(|| create_challenge());
    let chall = chall.await?;
    let prev = match prev.await {
        Ok(x) => x,
        Err(_) => return Err(errors::TaskError::SenderDropped),
    };
    let (mut rpid, mut hmac_req, mut salt) = ([0u8; 16], [0u8; 32], [0u8; BLOCK_SIZE - 16 - 32]);
    rpid.clone_from_slice(&prev[0..16]);
    hmac_req.clone_from_slice(&prev[16..(16 + 32)]);
    salt.clone_from_slice(&prev[(16 + 32)..]);
    let mut s = String::new();
    URL_SAFE.encode_string(rpid, &mut s);
    let rpid_str = format!("passchain-{s}");
    let dev = match dev.await {
        Ok(x) => x,
        Err(_) => return Err(errors::TaskError::SenderDropped),
    };
    let device_name = dev.device_name.clone();
    let (pwd_tx, pwd_rx) = new_string_channel();
    let pin_content = (format!("Enter FIDO PIN for \"{device_name}\":"), pwd_tx);
    if let Err(_) = pwd.send(pin_content).await {
        return Err(errors::TaskError::ReceiverDropped);
    }
    let pin = match pwd_rx.await {
        Ok(x) => Arc::new(x),
        Err(e) => return Err(errors::TaskError::FidoError(e.into())),
    };
    let pin_2 = pin.clone();
    let rpid_str_2 = rpid_str.clone();
    let dev = tokio::task::spawn_blocking(move || dev.open()).await?;
    let dev = match dev {
        Ok(x) => x,
        Err(e) => return Err(errors::TaskError::FidoError(e)),
    };
    tracing::warn!("Creating credential \"{rpid_str}\" on \"{device_name}\", please confirm.");
    let (dev, make_credential_result) = tokio::task::spawn_blocking(move || {
        let make_credential_args = MakeCredentialArgsBuilder::new(&rpid_str, &chall)
            .extensions(&[CredentialExtension::HmacSecret(Some(true))])
            .pin(&pin)
            .resident_key()
            .build();
        let make_credential_result = dev.make_credential_with_args(&make_credential_args);
        (dev, make_credential_result)
    })
    .await?;
    let make_credential_result = match make_credential_result {
        Ok(x) => x,
        Err(e) => return Err(errors::TaskError::FidoError(e)),
    };
    let chall_2 = chall_2.await?;
    let cid = make_credential_result.credential_descriptor.id;
    tracing::warn!("Generating hash for \"{rpid_str_2}\" on \"{device_name}\", please confirm.");
    // dev.credential_management_enumerate_credentials(pin);
    let (_, get_assertion_result) = tokio::task::spawn_blocking(move || {
        let a = GetAssertionArgsBuilder::new(&rpid_str_2, &chall_2)
            .credential_id(&cid)
            .extensions(&[AssertionExtension::HmacSecret(Some(hmac_req))])
            .pin(&pin_2)
            .build();
        let get_assertion_result = dev.get_assertion_with_args(&a);
        (dev, get_assertion_result)
    })
    .await?;
    let get_assertion_result = match get_assertion_result {
        Ok(x) => x,
        Err(e) => return Err(errors::TaskError::FidoError(e)),
    };
    if get_assertion_result.is_empty() {
        return Err(errors::TaskError::NoAssertionFound);
    }
    if get_assertion_result.len() != 1 {
        return Err(errors::TaskError::MultipleAssertionFound);
    }
    let assertion = get_assertion_result[0].clone();
    let mut cid = assertion.credential_id;
    let mut hmac_resp = [0u8; 32];
    for ext in assertion.extensions {
        let x = match ext {
            AssertionExtension::HmacSecret(x) => x,
            _ => continue,
        };
        if let Some(x) = x {
            hmac_resp.clone_from_slice(&x);
            break;
        }
    }
    if entropy::shannon_entropy(hmac_resp.clone()) <= 1.0 {
        return Err(errors::TaskError::LowEntropy);
    }
    let hasher = new_hasher();
    let mut pwd = vec![];

    pwd.append(&mut cid);
    pwd.append(&mut hmac_resp.to_vec());
    let out = tokio::task::spawn_blocking(move || {
        let mut out = new_block();
        match hasher.hash_password_into(&pwd, &salt, &mut out) {
            Ok(_) => Ok(out),
            Err(e) => Err(errors::TaskError::HasherError(e)),
        }
    })
    .await??;
    tracing::debug!("fido_factor_task done.");
    res.send(out).unwrap();
    Ok(())
}
