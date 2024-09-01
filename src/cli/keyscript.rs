// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::env;

use crate::errors;

pub struct Executor {}

impl Executor {
    pub async fn execute(self) -> anyhow::Result<(), errors::PasschainError> {
        let crypttab_name = env::var("CRYPTTAB_NAME").unwrap_or_default();
        let crypttab_source = env::var("CRYPTTAB_SOURCE").unwrap_or_default();
        let crypttab_key = env::var("CRYPTTAB_KEY").unwrap_or_default();
        let crypttab_options = env::var("CRYPTTAB_OPTIONS").unwrap_or_default();
        let crypttab_tried = env::var("CRYPTTAB_tried").unwrap_or_default();
        /*
          keyscript=<path>
        The executable at the indicated path is executed with the value of the third field as only argument. The keyscript's standard output is passed to cryptsetup as decyption key. Its exit status is currently ignored, but no assumption should be made in that regard. When used in initramfs, the executable either needs to be self-contained (i.e. doesn't rely on any external program which is not present in the initramfs environment) or the dependencies have to added to the initramfs image by other means. The program is either specified by full path or relative to /lib/cryptsetup/scripts/.

        LIMITATIONS: All binaries and files on which the keyscript depends must be available at the time of execution. Special care needs to be taken for encrypted filesystems like /usr or /var. As an example, unlocking encrypted /usr must not depend on binaries from /usr/(s)bin.

        This option is specific to the Debian crypttab format. It's not supported by systemd.

        WARNING: With systemd as init system, this option might be ignored. At the time this is written (December 2016), the systemd cryptsetup helper doesn't support the keyscript option to /etc/crypttab. For the time being, the only option to use keyscripts along with systemd is to force processing of the corresponding crypto devices in the initramfs. See the 'initramfs' option for further information.

        All fields of the appropriate crypttab entry are available to the keyscript as exported environment variables:

        CRYPTTAB_NAME, _CRYPTTAB_NAME
        The target name (after resp. before octal sequence decoding).

        CRYPTTAB_SOURCE, _CRYPTTAB_SOURCE
        The source device (after resp. before octal sequence decoding and device resolution).

        CRYPTTAB_KEY, _CRYPTTAB_KEY
        The value of the third field (after resp. before octal sequence decoding).

        CRYPTTAB_OPTIONS, _CRYPTTAB_OPTIONS
        A list of exported crypttab options (after resp. before octal sequence decoding).

        CRYPTTAB_OPTION_<option>
        The value of the appropriate crypttab option, with value set to 'yes' in case the option is merely a flag. For option aliases, such as 'readonly' and 'read-only', the variable name refers to the first alternative listed (thus 'CRYPTTAB_OPTION_readonly' in that case). If the crypttab option name contains '-' characters, then they are replaced with '_' in the exported variable name. For instance, the value of the 'CRYPTTAB_OPTION_keyfile_offset' environment variable is set to the value of the 'keyfile-offset' crypttab option.

        CRYPTTAB_TRIED
        Number of previous tries since start of cryptdisks (counts until maximum number of tries is reached).

         */
        Ok(())
    }
}
