// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use ctap_hid_fido2::{FidoKeyHid, HidInfo, fidokey::get_info::InfoParam};
use tokio::sync::oneshot;

use crate::{
    factor::{FactorOutputError, macros::TakeFinalValue},
    impl_statement_conversions,
    types::Block,
};

use super::{
    Factor, Mode, Question, QuestionProps, QuestionResponse, QuestionResponseError,
    QuestionResponseType, RespChan, macros::MaybeMut,
};

pub struct FidoKeyFactor {
    mode: Mode,
    block: Block,
    state: FidoKeyFactorStatement,
}

fn list_devices() -> Vec<FidoDevice> {
    let devices = ctap_hid_fido2::get_fidokey_devices()
        .iter()
        .map(|handler| FidoDevice::new(handler.clone()))
        .collect();
    devices
}

fn open_deivce(dev: &FidoDevice) -> Result<FidoKeyHid, anyhow::Error> {
    let cfg = ctap_hid_fido2::Cfg::init();
    let handler = dev.handler();
    let dev = ctap_hid_fido2::FidoKeyHidFactory::create_by_params(&[handler.param.clone()], &cfg);
    dev
}

impl FidoKeyFactor {
    fn next_question_create(&mut self) -> Result<Option<Question>, QuestionResponseError> {
        let state = FidoKeyFactorCreateStatement::maybe_mut(&mut self.state)
            .ok_or(QuestionResponseError::IncorrectUsage)?;
        match state {
            FidoKeyFactorCreateStatement::Start => {
                let devices = list_devices();
                let q = if devices.is_empty() {
                    let (q, chan) = Question::new(
                        "No FIDO Key found, would you like to retry?".into(),
                        QuestionProps::Confirm { default: true },
                    );
                    *state = FidoKeyFactorCreateStatement::Retry(Some(chan));
                    q
                } else {
                    let device_names = devices.iter().map(|x| x.name().to_string()).collect();
                    let (q, chan) = Question::new(
                        "Choose a FIDO Key to continue".into(),
                        QuestionProps::SingleSelect { list: device_names },
                    );
                    *state = FidoKeyFactorCreateStatement::WaitChooseDevice(Some(chan), devices);
                    q
                };
                Ok(Some(q))
            }
            FidoKeyFactorCreateStatement::Retry(_chan) => {
                // TODO: handle `No` selection
                let (q, _chan) = Question::new(
                    "Scaning devices...".into(),
                    QuestionProps::Continue { print_title: true },
                );
                *state = FidoKeyFactorCreateStatement::Start;
                Ok(Some(q))
            }
            FidoKeyFactorCreateStatement::WaitChooseDevice(chan, devices) => {
                // chan 这个时候肯定不是 None
                let chan = chan.take().unwrap();
                // 只要 sender 发送了数据, 那这边肯定不会出事的
                let resp = chan.blocking_recv().unwrap()?;
                let device = match resp {
                    QuestionResponse::SingleSelect(s) => {
                        let selected_device = &devices[s];
                        open_deivce(selected_device)
                    }
                    _ => return Err(QuestionResponseError::IncorrectUsage),
                };
                let mut device = match device {
                    Ok(d) => d,
                    Err(e) => {
                        let (q, chan) = Question::new(
                            format!("Error: {e:#}. Would you like to retry?"),
                            QuestionProps::Confirm { default: true },
                        );
                        *state = FidoKeyFactorCreateStatement::Retry(Some(chan));
                        return Ok(Some(q));
                    }
                };
                let version_detect = device.enable_info_param(&InfoParam::VersionsFido21);
                match version_detect {
                    Ok(x) => {
                        device.use_pre_credential_management = !x;
                    }
                    Err(e) => {
                        let (q, chan) = Question::new(
                            format!("Failed to get info: {e:#}. Would you like to retry?"),
                            QuestionProps::Confirm { default: true },
                        );
                        *state = FidoKeyFactorCreateStatement::Retry(Some(chan));
                        return Ok(Some(q));
                    }
                }
                let hmac_ext_detect = device.enable_info_param(&InfoParam::ExtensionsHmacSecret);
                match hmac_ext_detect {
                    Ok(x) => {
                        if !x {
                            let (q, chan) = Question::new(
                                "This device does not support HMAC-Secret Extension.".into(),
                                QuestionProps::Notice {},
                            );
                            *state = FidoKeyFactorCreateStatement::Retry(Some(chan));
                            return Ok(Some(q));
                        }
                    }
                    Err(e) => {
                        let (q, chan) = Question::new(
                            format!("Failed to get info: {e:#}. Would you like to retry?"),
                            QuestionProps::Confirm { default: true },
                        );
                        *state = FidoKeyFactorCreateStatement::Retry(Some(chan));
                        return Ok(Some(q));
                    }
                }
                let (q, chan) = Question::new(
                    "Input FIDO PIN".into(),
                    QuestionProps::Password {
                        allow_toggle: true,
                        confirm: true,
                    },
                );
                *state = FidoKeyFactorCreateStatement::RequestPIN(Some(chan), device);
                Ok(Some(q))
            }
            FidoKeyFactorCreateStatement::RequestPIN(chan, device) => {
                let chan = chan.take().unwrap();
                let resp = chan.blocking_recv().unwrap()?;
                let pin = match resp {
                    QuestionResponse::Password(s) => s,
                    _ => return Err(QuestionResponseError::IncorrectUsage),
                };
                let rpid = "passchain-xxxxxxxxxxxxxxxxxxxx"; // max 32 bytes
                // TODO
                // device.get_pin_token(cid, pin);
                // device.credential_management_enumerate_credentials(pin, rpid_hash);
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    // fn next_question_verify(&mut self) -> Result<Option<Question>, QuestionResponseError> {
    //     let state = FidoKeyFactorVerifyStatement::maybe_mut(&mut self.state)
    //         .ok_or(QuestionResponseError::IncorrectUsage)?;
    //     match state {
    //         FidoKeyFactorVerifyStatement::Start => {
    //             let (q, chan) = Question::new(
    //                 "Input password".into(),
    //                 QuestionProps::Password {
    //                     allow_toggle: true,
    //                     confirm: false,
    //                 },
    //             );
    //             *state = FidoKeyFactorVerifyStatement::Wait(Some(chan));
    //             Ok(Some(q))
    //         }
    //         FidoKeyFactorVerifyStatement::Wait(chan) => {
    //             // chan 这个时候肯定不是 None
    //             let chan = chan.take().unwrap();
    //             // 只要 sender 发送了数据, 那这边肯定不会出事的
    //             let password = chan.blocking_recv().unwrap()?;
    //             match password {
    //                 QuestionResponse::Password(p) => {
    //                     *state = FidoKeyFactorVerifyStatement::Final(Some(p));
    //                 }
    //                 _ => {
    //                     return Err(QuestionResponseError::IncorrectUsage);
    //                 }
    //             }
    //             Ok(None)
    //         }
    //         _ => Ok(None),
    //     }
    // }
}

impl Factor for FidoKeyFactor {
    fn new(block: Block, mode: Mode) -> Self {
        Self {
            mode,
            block,
            state: mode.into(),
        }
    }

    fn next_question(&mut self) -> Result<Option<Question>, QuestionResponseError> {
        todo!()
    }

    fn result(mut self) -> Result<Block, super::FactorOutputError> {
        let password = self
            .state
            .take_final()
            .ok_or(FactorOutputError::IncorrectUsage)?;
        todo!()
    }
}

pub enum FidoKeyFactorStatement {
    Create(FidoKeyFactorCreateStatement),
    Verify(FidoKeyFactorVerifyStatement),
}

#[derive(Default)]
pub enum FidoKeyFactorCreateStatement {
    #[default]
    Start,
    Retry(RespChan),
    WaitChooseDevice(RespChan, Vec<FidoDevice>),
    RequestPIN(RespChan, FidoKeyHid),
    Final(Option<()>),
}

#[derive(Default)]
pub enum FidoKeyFactorVerifyStatement {
    #[default]
    Start,
    Final(Option<()>),
}

impl_statement_conversions!(
    FidoKeyFactorStatement,
    FidoKeyFactorCreateStatement,
    FidoKeyFactorVerifyStatement,
    Option<()>
);

pub struct FidoDevice {
    name: String,
    handler: HidInfo,
}

impl FidoDevice {
    pub fn new(handler: HidInfo) -> Self {
        let vid = hex::encode(handler.vid.to_be_bytes());
        let pid = hex::encode(handler.pid.to_be_bytes());
        let sn = handler
            .serial_number
            .as_ref()
            .map(|x| format!("SN:{x}"))
            .unwrap_or("SN:(none)".to_string());
        let path = match &handler.param {
            ctap_hid_fido2::HidParam::Path(s) => &format!("Path:{s}"),
            _ => "Path:(none)",
        };
        let s = format!("{vid}:{pid} {} {} {}", &handler.product_string, &sn, path);
        FidoDevice { name: s, handler }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn handler(&self) -> &HidInfo {
        &self.handler
    }
}

// impl std::fmt::Display for FidoDevice {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         write!(f, "{}", self.name)
//     }
// }
