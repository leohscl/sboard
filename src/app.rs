use crate::job_handler::{self, build_display, DisplayMode};
use crate::job_handler::{run_scontrol, run_squeue};
use crate::Cli;
use color_eyre::eyre::Result;
use color_eyre::Report;

pub enum DisplayState {
    Jobs,
    Details(JobDetails),
}

pub struct JobDetails {
    pub job_id: String,
    pub err_file: String,
    pub log_file: String,
}

pub struct App {
    cli: Cli,
    pub display_state: DisplayState,
    pub jobs: Option<Vec<String>>,
    pub highlighted: Option<usize>,
}

impl App {
    pub fn new(cli: Cli) -> App {
        App {
            cli,
            jobs: None,
            highlighted: None,
            display_state: DisplayState::Jobs,
        }
    }

    pub fn get_refresh(&self) -> bool {
        self.cli.refresh
    }

    pub fn get_display(&self) -> DisplayMode {
        self.cli.display_mode
    }

    fn get_jobs(&self) -> Result<Vec<String>> {
        self.jobs.clone().ok_or(Report::msg("jobs empty"))
    }

    pub fn fetch_job_info(&mut self) -> Result<()> {
        if let Some(highlighted_i) = self.highlighted {
            let job = &self.get_jobs()?[highlighted_i];
            let job_id = job_handler::parse_job_id(job)?;
            let job_info = run_scontrol(self.cli.run_mode, &job_id)?;
            let job_details = job_handler::parse_job_details(&job_id, &job_info)?;
            self.display_state = DisplayState::Details(job_details);
        }
        Ok(())
    }

    pub fn fetch_jobs(&mut self) -> Result<()> {
        // Only fetch results if needed
        let new_results = if self.get_refresh() || self.jobs.is_none() {
            let current_job_info = run_squeue(self.cli.run_mode)?;
            build_display(current_job_info, &self)?
        } else {
            self.get_jobs()?
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
        self.jobs = Some(new_results);
        Ok(())
    }

    pub fn send_char(&mut self, c_sent: char) -> Result<()> {
        match c_sent {
            't' => self.cli.refresh = !self.cli.refresh,
            'j' => self.increase_highlighted()?,
            'k' => self.decrease_highlighted()?,
            _ => (),
        }
        Ok(())
    }

    pub fn send_quit(&mut self) -> bool {
        match self.display_state {
            DisplayState::Details(_) => {
                self.display_state = DisplayState::Jobs;
                false
            }
            DisplayState::Jobs => true,
        }
    }

    fn offset_highlighted(&mut self, offset: i32) -> Result<()> {
        let num_results = self.get_jobs()?.len() as i32;
        if let Some(highlighted_i) = self.highlighted {
            assert!(highlighted_i != 0 && highlighted_i < num_results as usize);
            let new_value = (highlighted_i as i32 + offset - 1).rem_euclid(num_results - 1) + 1;
            self.highlighted = Some(new_value as usize);
        }
        Ok(())
    }

    fn decrease_highlighted(&mut self) -> Result<()> {
        self.offset_highlighted(-1)
    }

    fn increase_highlighted(&mut self) -> Result<()> {
        self.offset_highlighted(1)
    }
}
