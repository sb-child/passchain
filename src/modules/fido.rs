// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub enum Request {
    ListDevices(OnetimeTx<ListDevicesResponse>),
    Wink(Device, OnetimeTx<OnetimeRx<bool>>),
}

pub struct ListDevicesResponse {
    pub list: MultRx<Device>,
}

pub struct Device {
    name: String,
    id: ctap_hid_fido2::HidInfo,
}

impl Device {
    fn open(&self) -> anyhow::Result<ctap_hid_fido2::FidoKeyHid> {
        let current_param = self.id.param.clone();
        let dev = ctap_hid_fido2::FidoKeyHidFactory::create_by_params(
            &[current_param],
            &ctap_hid_fido2::Cfg::init(),
        );
        dev
    }
}

impl std::fmt::Display for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

fn new_cmd_pipe() -> Mult<Request> {
    new_mult()
}

fn new_mult<X>() -> Mult<X> {
    tokio::sync::mpsc::channel(1)
}

fn new_onetime<X>() -> Onetime<X> {
    tokio::sync::oneshot::channel()
}

type CommandPipeTx = MultTx<Request>;
type CommandPipeRx = MultRx<Request>;

type Onetime<X> = (OnetimeTx<X>, OnetimeRx<X>);
type OnetimeTx<X> = tokio::sync::oneshot::Sender<X>;
type OnetimeRx<X> = tokio::sync::oneshot::Receiver<X>;

type Mult<X> = (MultTx<X>, MultRx<X>);
type MultTx<X> = tokio::sync::mpsc::Sender<X>;
type MultRx<X> = tokio::sync::mpsc::Receiver<X>;

pub struct AsyncFido {}

impl AsyncFido {
    pub fn new() -> Self {
        new_cmd_pipe();
        AsyncFido {}
    }
}

fn event_handler(mut pipe: CommandPipeRx) {
    loop {
        let cmd = pipe.blocking_recv();
        let cmd = if let Some(cmd) = cmd { cmd } else { break };
        match cmd {
            Request::ListDevices(resp) => {
                let (tx, rx) = new_mult();
                tokio::task::spawn_blocking(|| list_device(tx));
                resp.send(ListDevicesResponse { list: rx });
            }
            Request::Wink(dev, resp) => {
                let (tx, rx) = new_onetime();
                tokio::task::spawn_blocking(|| wink(dev, tx));
                resp.send(rx);
            }
        }
    }
}

fn list_device(tx: MultTx<Device>) {
    let devs = ctap_hid_fido2::get_fidokey_devices();
    for info in devs {
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
                tx.blocking_send(Device {
                    name: n,
                    id: current_info,
                });
            }
            Err(e) => {
                tracing::error!("Unable to open the device {device_name}: {e}");
            }
        }
    }
}

fn wink(device: Device, tx: OnetimeTx<bool>) {
    let dev = device.open();
    let fido = match dev {
        Ok(x) => x,
        Err(e) => {
            tracing::error!("Unable to open the device {0}: {e}", device.name);
            tx.send(false);
            return;
        }
    };
    let wk = fido.wink();
    match wk {
        Ok(_) => {
            tx.send(true);
        }
        Err(e) => {
            tracing::error!("Failed to wink {0}: {e}", device.name);
            tx.send(false);
        }
    }
}
