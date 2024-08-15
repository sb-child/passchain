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

type TaskErrorSender = tokio::sync::oneshot::Sender<errors::TaskError>;
type TaskErrorReceiver = tokio::sync::oneshot::Receiver<errors::TaskError>;

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
    fn trigger_wink(&self) -> bool {
        let pb = ProgressBar::new(2);
        let current_param = self.info.param.clone();
        let dev = ctap_hid_fido2::FidoKeyHidFactory::create_by_params(
            &[current_param],
            &ctap_hid_fido2::Cfg::init(),
        );
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
                format!("{:#06x}:{:#06x}", vid, pid)
            }
            ctap_hid_fido2::HidParam::Path(x) => {
                format!("{:#06x}:{:#06x} ({})", info.vid, info.pid, x)
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
        dev: FidoReceiver,
        prev: BlockReceiver,
        res: BlockSender,
    },
}

impl Task {
    async fn run(err_rx: TaskErrorReceiver) {}
}

async fn nonce_task(nonce: BlockSender) -> anyhow::Result<(), errors::TaskError> {
    let x = tokio::task::spawn_blocking(|| new_random_block()).await?;
    if let Err(_) = nonce.send(x) {
        Err(errors::TaskError::SenderDropped)
    } else {
        Ok(())
    }
}
