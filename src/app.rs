use crate::job_handler::run_command;
use crate::job_handler::{build_display, DisplayMode};
use crate::Cli;

pub struct App {
    cli: Cli,
    pub results: Option<Vec<String>>,
    highlighted: Option<usize>,
}

impl App {
    pub fn new(cli: Cli) -> App {
        App {
            cli,
            results: None,
            highlighted: None,
        }
    }

    pub fn get_refresh(&self) -> bool {
        self.cli.refresh
    }

    pub fn get_display(&self) -> DisplayMode {
        self.cli.display_mode
    }

    // is this supposed to be in this file ?
    // split logic in ui file
    pub fn fetch_results(&mut self) {
        // Only fetch results if needed
        let new_results = if self.get_refresh() || self.results.is_none() {
            let squeue_args = vec!["--me", "--format=%all"];
            let current_job_info =
                run_command(&self.cli.run_mode, &squeue_args).expect("Failed running command");
            build_display(current_job_info, &self).expect("Failed building display")
        } else {
            self.results.clone().unwrap()
        };

        let new_highlight = if new_results.is_empty() || self.get_refresh() {
            None
        } else {
            if self.highlighted.is_none() {
                Some(0)
            } else {
                self.highlighted
            }
        };
        self.highlighted = new_highlight;
    }

    pub fn send_char(&mut self, c_sent: char) {
        match c_sent {
            't' => self.cli.refresh = !self.cli.refresh,
            _ => (),
        }
    }
}
