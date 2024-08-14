// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::errors;
use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressIterator};
use inquire::InquireError;
use strum::{Display, EnumDiscriminants, EnumIter, EnumString, IntoEnumIterator};

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
    pub async fn execute(self) -> anyhow::Result<(), errors::PasschainError> {
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
        self.prompt()
        // Ok(())
    }

    fn prompt(&self) -> anyhow::Result<(), errors::PasschainError> {
        'outer: for i in 1..u32::MAX {
            'inner: loop {
                let fa = match FactorDiscriminants::ask(i) {
                    Ok(f) => Factor::ask(f),
                    Err(e) => match e {
                        e @ errors::AskError::InquireError(_) => {
                            return Err(errors::PasschainError::AskError(e))
                        }
                        errors::AskError::Interrupted => {
                            return Err(errors::PasschainError::AskError(e))
                        }
                        errors::AskError::Canceled => break 'outer,
                    },
                };
                match fa {
                    Ok(f) => {
                        let fac_name = FactorDiscriminants::from(f);
                        tracing::info!("Factor {i} with {fac_name} added.");
                    }
                    Err(e) => match e {
                        e @ errors::AskError::InquireError(_) => {
                            return Err(errors::PasschainError::AskError(e))
                        }
                        errors::AskError::Interrupted => {
                            return Err(errors::PasschainError::AskError(e))
                        }
                        errors::AskError::Canceled => continue 'inner,
                    },
                };
                continue 'outer;
            }
        };
        Ok(())
    }
}

struct FidoList {
    info: ctap_hid_fido2::HidInfo,
    device_name: String,
}

impl std::fmt::Display for FidoList {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.device_name)
    }
}

fn get_fido_list() -> Vec<FidoList> {
    let devs = ctap_hid_fido2::get_fidokey_devices();
    let mut r = vec![];
    for info in devs.into_iter().progress_with(ProgressBar::new(20)) {
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
                r.push(FidoList {
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

#[derive(EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter, EnumString, Display))]
enum Factor {
    Password(String),
    Fido(FidoList),
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
                todo!()
            }
            FactorDiscriminants::Fido => {
                let list = get_fido_list();
                let ans: Result<FidoList, InquireError> =
            inquire::Select::new(&format!("Select the FIDO key to store this factor:"), list)
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
