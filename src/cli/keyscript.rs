// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{env, io::stderr};

use ratatui::{
    crossterm::{
        event::{self, KeyCode, KeyEventKind},
        execute,
        terminal::{
            self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
        },
    },
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Paragraph},
    DefaultTerminal, Terminal,
};

use crate::{config::Cfg, errors};

pub struct Executor {
    cfg: Cfg,
    source_name: String,
    target_name: String,
    /// 0 means infinitive retries
    max_try: u32,
    tried: u32,
}

impl Executor {
    pub fn new(cfg_path: String) -> anyhow::Result<Self, errors::PasschainError> {
        tracing::info!("Config file path: {}", cfg_path);

        let cfg_path = std::fs::canonicalize(cfg_path)
            .map_err(|x| errors::PasschainError::CanonicalizeError(x))?;

        tracing::info!("Loading config file: {:?}", cfg_path);

        let crypttab_name = env::var("CRYPTTAB_NAME").unwrap_or("UNKNOWN".into());
        let crypttab_source = env::var("CRYPTTAB_SOURCE").unwrap_or("UNKNOWN".into());
        let crypttab_key = env::var("CRYPTTAB_KEY").unwrap_or_default();
        let crypttab_options = env::var("CRYPTTAB_OPTIONS").unwrap_or_default();
        let crypttab_option_tries = env::var("CRYPTTAB_OPTION_tries").unwrap_or("3".into());
        let crypttab_tried = env::var("CRYPTTAB_TRIED").unwrap_or_default();

        let cfg = Cfg::load(cfg_path)?;
        Ok(Self {
            cfg,
            source_name: crypttab_source,
            target_name: crypttab_name,
            max_try: crypttab_option_tries.parse().unwrap_or(0),
            tried: crypttab_tried.parse().unwrap_or(0),
        })
    }
    pub async fn execute(mut self) -> anyhow::Result<(), errors::PasschainError> {
        tracing::info!("Enter the terminal ui...");

        let backend = ratatui::backend::CrosstermBackend::new(stderr());
        let mut terminal = ratatui::Terminal::with_options(
            backend,
            ratatui::TerminalOptions {
                viewport: ratatui::Viewport::Fullscreen,
            },
        )
        .map_err(|x| errors::PasschainError::TuiError(x))?;
        enable_raw_mode().map_err(|x| errors::PasschainError::TuiError(x))?;
        execute!(stderr(), EnterAlternateScreen)
            .map_err(|x| errors::PasschainError::TuiError(x))?;
        terminal
            .clear()
            .map_err(|x| errors::PasschainError::TuiError(x))?;
        let app_result = self.ui(terminal);
        disable_raw_mode().map_err(|x| errors::PasschainError::TuiError(x))?;
        execute!(stderr(), LeaveAlternateScreen)
            .map_err(|x| errors::PasschainError::TuiError(x))?;
        tracing::info!("Terminal quited.");

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
        // Ok(())
        app_result
    }

    fn ui<T: ratatui::backend::Backend>(
        &mut self,
        mut terminal: Terminal<T>,
    ) -> anyhow::Result<(), errors::PasschainError> {
        fn short(x: &str) -> String {
            if x.len() > 13 {
                let x = &x[10..];
                format!("...{}", x)
            } else {
                format!("{}", x)
            }
        }

        let device_str = {
            let source_name = short(&self.source_name);
            let target_name = short(&self.target_name);
            format!("{} -> {}", source_name, target_name)
        };
        let tries_str = if self.max_try == 0 {
            format!("{} tries", self.tried)
        } else {
            format!("{}/{} chances used", self.tried, self.max_try)
        };

        loop {
            terminal
                .draw(|frame| {
                    let block = Block::bordered()
                        .title_top(
                            Line::from("PassChain")
                                .centered()
                                .style(Style::new().bg(Color::White).fg(Color::Black)),
                        )
                        .title_top(
                            Line::from("01")
                                .left_aligned()
                                .style(Style::new().fg(Color::Red)),
                        )
                        .title_top(
                            Line::from("Fido")
                                .right_aligned()
                                .style(Style::new().fg(Color::Green)),
                        )
                        .title_bottom(Line::from(device_str.clone()).left_aligned())
                        .title_bottom(Line::from(tries_str.clone()).right_aligned());
                    let centered_frame =
                        center(frame.area(), Constraint::Max(60), Constraint::Max(10));
                    let inner = block.inner(frame.area());

                    frame.render_widget(block, centered_frame);
                    // frame.render_widget(centered_area, frame.area());
                })
                .map_err(|x| errors::PasschainError::TuiError(x))?;

            if let event::Event::Key(key) =
                event::read().map_err(|x| errors::PasschainError::TuiError(x))?
            {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    return Ok(());
                }
            }
        }

        // Ok(())
    }
}

fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}
