use crate::display::build_display;
use crate::job_handler::run_command;
use crate::Cli;

pub struct App {
    cli: Cli,
    results: Option<Vec<String>>,
}

impl App {
    pub fn new(cli: Cli) -> App {
        App { cli, results: None }
    }

    pub fn get_refresh(&self) -> bool {
        self.cli.refresh
    }

    pub fn make_ui(&mut self) -> Vec<String> {
        if self.get_refresh() || self.results.is_none() {
            let squeue_args = vec!["--me", "--format=%all"];
            let current_job_info =
                run_command(&self.cli.run_mode, &squeue_args).expect("Failed running command");
            self.results = Some(
                build_display(current_job_info, &self.cli.display_mode)
                    .expect("Failed building display"),
            );
        }
        self.results.clone().unwrap()
    }

    pub fn send_char(&mut self, c_sent: char) {
        match c_sent {
            't' => self.cli.refresh = !self.cli.refresh,
            _ => (),
        }
    }
}
