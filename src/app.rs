use crate::job_handler::run_command;
use crate::job_handler::{build_display, DisplayMode};
use crate::Cli;

pub struct App {
    cli: Cli,
    pub results: Option<Vec<String>>,
    pub highlighted: Option<usize>,
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
                if new_results.len() >= 2 {
                    Some(1)
                } else {
                    None
                }
            } else {
                self.highlighted
            }
        };
        // update state
        self.highlighted = new_highlight;
        self.results = Some(new_results);
    }

    pub fn send_char(&mut self, c_sent: char) {
        match c_sent {
            't' => self.cli.refresh = !self.cli.refresh,
            'j' => self.increase_highlighted(),
            'k' => self.decrease_highlighted(),
            _ => (),
        }
    }

    fn decrease_highlighted(&mut self) {
        let num_results = self.results.clone().unwrap().len();
        if let Some(highlighted_i) = self.highlighted {
            assert!(highlighted_i != 0 && highlighted_i < num_results);
            let new_highlighted_i = if highlighted_i == 1 {
                num_results - 1
            } else {
                highlighted_i - 1
            };
            self.highlighted = Some(new_highlighted_i);
        }
    }

    fn increase_highlighted(&mut self) {
        let num_results = self.results.clone().unwrap().len();
        if let Some(highlighted_i) = self.highlighted {
            assert!(highlighted_i != 0 && highlighted_i < num_results);
            let new_highlighted_i = if highlighted_i == num_results - 1 {
                1
            } else {
                highlighted_i + 1
            };
            self.highlighted = Some(new_highlighted_i);
        }
    }
}
