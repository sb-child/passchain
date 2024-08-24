// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::errors;
use clap::{Parser, Subcommand};

use indicatif::{ProgressBar, ProgressIterator};
use inquire::InquireError;

use strum::{Display, EnumDiscriminants, EnumIter, EnumString, IntoEnumIterator};

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
        argon2::Params::new(1000000, 30, 1, Some(BLOCK_SIZE)).unwrap(),
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
        // let devs = ctap_hid_fido2::get_fidokey_devices();
        // for info in devs {
        //     println!("\n\n---------------------------------------------");
        //     println!(
        //         "- vid=0x{:04x} , pid=0x{:04x} , info={:?}",
        //         info.vid, info.pid, info.info
        //     );

        //     let dev = ctap_hid_fido2::FidoKeyHidFactory::create_by_params(
        //         &[info.param],
        //         &ctap_hid_fido2::Cfg::init(),
        //     )
        //     .unwrap();
        //     dev.wink().unwrap();
        //     println!("get_info()");
        //     match dev.get_info() {
        //         Ok(info) => println!("{}", info),
        //         Err(e) => println!("error: {:?}", e),
        //     }

        //     println!("get_pin_retries()");
        //     match dev.get_pin_retries() {
        //         Ok(info) => println!("{}", info),
        //         Err(e) => println!("error: {:?}", e),
        //     }

        //     println!("get_info_u2f()");
        //     match dev.get_info_u2f() {
        //         Ok(info) => println!("{}", info),
        //         Err(e) => println!("error: {:?}", e),
        //     }

        //     println!("enable_info_option() - ClinetPin");
        //     match dev.enable_info_option(&ctap_hid_fido2::fidokey::get_info::InfoOption::ClientPin)
        //     {
        //         Ok(result) => println!("PIN = {:?}", result),
        //         Err(e) => println!("- error: {:?}", e),
        //     }
        // }
        let prompt_thread = tokio::task::spawn_blocking(|| prompt_factors());
        let factors = prompt_thread.await??;
        self.compute(factors).await?;
        Ok(())
    }

    async fn compute(
        &mut self,
        factors: Vec<Factor>,
    ) -> anyhow::Result<(), errors::PasschainError> {
        
        Ok(())
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
        // let argon2 = argon2::Argon2::new(
        //     argon2::Algorithm::Argon2id,
        //     argon2::Version::V0x13,
        //     argon2::Params::new(1000000, 30, 1, Some(128)).unwrap(),
        // );
        // let mut out = [0u8; 128];
        // let pwd = [1u8; 128];
        // let salt = [3u8; 128];
        // tracing::info!("calc");
        // argon2.hash_password_into(&pwd, &salt, &mut out).unwrap();
        // tracing::info!("result: {:?}", out);
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
        // let challenge = ctap_hid_fido2::verifier::create_challenge();
        // let hm = ctap_hid_fido2::verifier::create_challenge();
        // // let make_credential_args = ctap_hid_fido2::fidokey::MakeCredentialArgsBuilder::new(rpid, &challenge)
        // // .pin(&pin)
        // // .build();
        // let argon2 = argon2::Argon2::default();
        // // argon2.hash_password_into(pwd, salt, out);
        // let a = ctap_hid_fido2::fidokey::GetAssertionArgsBuilder::new("", &challenge)
        //     .extensions(&[ctap_hid_fido2::fidokey::AssertionExtension::HmacSecret(
        //         Some(hm),
        //     )])
        //     .build();
        // let device =
        //     ctap_hid_fido2::FidoKeyHidFactory::create(&ctap_hid_fido2::Cfg::init()).unwrap();
        // // let info = device.get_info().unwrap();
        // let x = device.get_assertion_with_args(&a).unwrap();
        // for y in x {
        //     y.credential_id;
        //     for z in y.extensions {
        //         match z {
        //             ctap_hid_fido2::fidokey::AssertionExtension::HmacSecret(x) => todo!(),
        //             ctap_hid_fido2::fidokey::AssertionExtension::LargeBlobKey(_) => todo!(),
        //             ctap_hid_fido2::fidokey::AssertionExtension::CredBlob(_) => todo!(),
        //         }
        //     }
        // }
        // None
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
    async fn run(self, err_tx: TaskErrorSender) {
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
        }
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
    res.send(out).unwrap();
    Ok(())
}

async fn fido_factor_task(
    pwd: AskPinSender,
    dev: FidoReceiver,
    prev: BlockReceiver,
    res: BlockSender,
) -> anyhow::Result<(), errors::TaskError> {
    use ctap_hid_fido2::{
        fidokey::{
            AssertionExtension, CredentialExtension, GetAssertionArgsBuilder,
            MakeCredentialArgsBuilder,
        },
        verifier::create_challenge,
    };
    use std::sync::Arc;
    use tokio::sync::Mutex;
    let chall = tokio::task::spawn_blocking(|| create_challenge());
    let chall_2 = tokio::task::spawn_blocking(|| create_challenge());
    let mut device_name = "".to_string();
    let dev = match dev.await {
        Ok(x) => {
            device_name = x.device_name.clone();
            let x = tokio::task::spawn_blocking(move || x.open());
            x.await?
        }
        Err(_) => return Err(errors::TaskError::SenderDropped),
    };
    let dev = match dev {
        Ok(x) => x,
        Err(e) => return Err(errors::TaskError::FidoError(e)),
    };
    let (pwd_tx, pwd_rx) = new_string_channel();
    let pin_content: AskPinContent = (format!("Enter FIDO PIN for \"{device_name}\":"), pwd_tx);
    if let Err(_) = pwd.send(pin_content).await {
        return Err(errors::TaskError::ReceiverDropped);
    }
    let chall = chall.await?;
    let prev = match prev.await {
        Ok(x) => x,
        Err(_) => return Err(errors::TaskError::SenderDropped),
    };
    let (mut rpid, mut hmac_req, mut salt) = ([0u8; 16], [0u8; 32], [0u8; BLOCK_SIZE - 16 - 32]);
    rpid.clone_from_slice(&prev[0..16]);
    hmac_req.clone_from_slice(&prev[16..(16 + 32)]);
    salt.clone_from_slice(&prev[(16 + 32)..]);
    let rpid_str = format!("passchain-{}", hex::encode(rpid));
    let pin = match pwd_rx.await {
        Ok(x) => Arc::new(x),
        Err(e) => return Err(errors::TaskError::FidoError(e.into())),
    };
    let pin_2 = pin.clone();
    let rpid_str_2 = rpid_str.clone();
    tracing::warn!("Creating credential \"{rpid_str}\" on \"{device_name}\", please comfirm.");
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
    let cid = make_credential_result.rpid_hash;
    tracing::warn!("Generating hash for \"{rpid_str_2}\" on \"{device_name}\", please comfirm.");
    let (dev, get_assertion_result) = tokio::task::spawn_blocking(move || {
        let a = GetAssertionArgsBuilder::new(&rpid_str_2, &chall_2)
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
    let mut hmac_resp: Vec<u8> = vec![];
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
    pwd.append(&mut hmac_resp);
    let out = tokio::task::spawn_blocking(move || {
        let mut out = new_block();
        match hasher.hash_password_into(&pwd, &salt, &mut out) {
            Ok(_) => Ok(out),
            Err(e) => Err(errors::TaskError::HasherError(e)),
        }
    })
    .await??;
    res.send(out).unwrap();
    Ok(())
}
