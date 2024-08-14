// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use clap::{Parser, Subcommand};
use inquire::InquireError;
use strum::{Display, EnumDiscriminants, EnumIter, EnumString, IntoEnumIterator};

use crate::errors;

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
        let devs = ctap_hid_fido2::get_fidokey_devices();
        for info in devs {
            println!("\n\n---------------------------------------------");
            println!(
                "- vid=0x{:04x} , pid=0x{:04x} , info={:?}",
                info.vid, info.pid, info.info
            );

            let dev = ctap_hid_fido2::FidoKeyHidFactory::create_by_params(
                &[info.param],
                &ctap_hid_fido2::Cfg::init(),
            )
            .unwrap();

            println!("get_info()");
            match dev.get_info() {
                Ok(info) => println!("{}", info),
                Err(e) => println!("error: {:?}", e),
            }

            println!("get_pin_retries()");
            match dev.get_pin_retries() {
                Ok(info) => println!("{}", info),
                Err(e) => println!("error: {:?}", e),
            }

            println!("get_info_u2f()");
            match dev.get_info_u2f() {
                Ok(info) => println!("{}", info),
                Err(e) => println!("error: {:?}", e),
            }

            println!("enable_info_option() - ClinetPin");
            match dev.enable_info_option(&ctap_hid_fido2::fidokey::get_info::InfoOption::ClientPin)
            {
                Ok(result) => println!("PIN = {:?}", result),
                Err(e) => println!("- error: {:?}", e),
            }
        }
        FactorDiscriminants::ask(1);
        FactorDiscriminants::ask(2);
        FactorDiscriminants::ask(3);
        FactorDiscriminants::ask(4);
        Ok(())
    }

    fn prompt(&self) {}
}

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter, EnumString, Display))]
enum Factor {
    Password(String),
    Fido(),
}

impl Factor {
    fn ask(selected: FactorDiscriminants) -> Option<Self> {
        let challenge = ctap_hid_fido2::verifier::create_challenge();
        // let make_credential_args = ctap_hid_fido2::fidokey::MakeCredentialArgsBuilder::new(rpid, &challenge)
        // .pin(&pin)
        // .build();

        None
    }
}

impl FactorDiscriminants {
    fn ask(n: u32) -> Option<Self> {
        let options: Vec<_> = FactorDiscriminants::iter().collect();
        let ans: Result<FactorDiscriminants, InquireError> =
            inquire::Select::new(&format!("[{n}] Select a factor:"), options)
                .with_help_message("Be sure to remember the order of the factors, otherwise you will not be able to decrypt it later.")
                .prompt();
        match ans {
            Ok(choice) => {
                // println!("{}", choice);
                Some(choice)
            }
            Err(_) => {
                // println!("There was an error, please try again");
                None
            }
        }
    }
}
