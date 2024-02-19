use crate::editor::Editor;
use crate::job_handler;
use crate::job_handler::DisplayMode;
use crate::job_handler::{run_scontrol, run_squeue};
use crate::parser::RunMode;
use crate::Cli;
use color_eyre::eyre::Result;
use color_eyre::Report;

pub enum DisplayState<'a> {
    Empty,
    Jobs(JobList),
    Details(JobDetails),
    Editor(Editor<'a>),
}

pub struct JobList {
    pub jobs: Vec<String>,
}

pub struct JobDetails {
    pub job_id: String,
    pub err_file: String,
    pub log_file: String,
}

pub struct App<'a> {
    cli: Cli,
    pub display_state: DisplayState<'a>,
    pub highlighted: Option<usize>,
}

impl<'a> App<'a> {
    pub fn new(cli: Cli) -> App<'a> {
        App {
            cli,
            highlighted: None,
            display_state: DisplayState::Empty,
        }
    }

    pub fn get_refresh(&self) -> bool {
        self.cli.refresh
    }

    pub fn get_display(&self) -> DisplayMode {
        self.cli.display_mode
    }

    fn get_jobs(&self) -> Result<Vec<String>> {
        match self.display_state {
            DisplayState::Jobs(ref job_list) => Ok(job_list.jobs.clone()),
            _ => panic!("Get jobs called without jobs displayed"),
        }
    }

    pub fn send_enter(&mut self) -> Result<()> {
        match self.display_state {
            DisplayState::Empty => Ok(()),
            DisplayState::Jobs(_) => self.fetch_job_info(),
            DisplayState::Details(ref details) => {
                let logs = Self::fetch_logs(self.cli.run_mode, details)?;
                self.display_state = DisplayState::Editor(Editor::new(&logs));
                Ok(())
            }
            DisplayState::Editor(_) => todo!(),
        }
    }

    fn fetch_logs(run_mode: RunMode, details: &JobDetails) -> Result<String> {
        // TODO(lhenches): use highlighted_i
        job_handler::read_file(run_mode, &details.log_file)
    }

    fn fetch_job_info(&mut self) -> Result<()> {
        if let DisplayState::Jobs(_) = self.display_state {
            if let Some(highlighted_i) = self.highlighted {
                let job = &self.get_jobs()?[highlighted_i];
                let job_id = job_handler::parse_job_id(job)?;
                let job_info = run_scontrol(self.cli.run_mode, &job_id)?;
                let job_details = job_handler::parse_job_details(&job_id, &job_info)?;
                self.display_state = DisplayState::Details(job_details);
                self.highlighted = Some(0);
            }
        } else {
            return Err(Report::msg(
                "fetch_job_info called when no jobs are displayed",
            ));
        }
        Ok(())
    }

    pub fn fetch_jobs(&mut self) -> Result<()> {
        // Only fetch results if needed
        if self.get_refresh() || matches!(self.display_state, DisplayState::Empty) {
            let current_job_info = run_squeue(self.cli.run_mode)?;
            let new_results = job_handler::build_display(current_job_info, self)?;
            self.highlighted = if new_results.len() >= 2 {
                Some(1)
            } else {
                None
            };
            self.display_state = DisplayState::Jobs(JobList { jobs: new_results });
        };
        Ok(())
    }

    pub fn send_char(&mut self, c_sent: char) -> Result<()> {
        match self.display_state {
            DisplayState::Editor(ref mut editor) => editor.send_char(c_sent),
            _ => match c_sent {
                't' => self.cli.refresh = !self.cli.refresh,
                'j' => self.increase_highlighted()?,
                'k' => self.decrease_highlighted()?,
                _ => (),
            },
        }
        Ok(())
    }

    pub fn send_quit(&mut self) -> bool {
        match self.display_state {
            DisplayState::Details(_) | DisplayState::Editor(_) => {
                self.highlighted = None;
                self.display_state = DisplayState::Empty;
                false
            }
            DisplayState::Jobs(_) | DisplayState::Empty => true,
        }
    }

    fn offset_highlighted(&mut self, offset: i32) -> Result<()> {
        match self.display_state {
            DisplayState::Jobs(ref job_list) => {
                let num_skip_line = 1;
                let num_results = job_list.jobs.len() as i32;
                self.offset_highlighted_with_params(offset, num_results, num_skip_line);
            }
            DisplayState::Details(_) => {
                let num_results = 2;
                let num_skip_line = 0;
                self.offset_highlighted_with_params(offset, num_results, num_skip_line);
            }
            DisplayState::Editor(_) => panic!("Cannot offset in editor mode"),
            DisplayState::Empty => (),
        }
        Ok(())
    }

    fn offset_highlighted_with_params(
        &mut self,
        offset: i32,
        num_results: i32,
        num_skip_line: i32,
    ) {
        if let Some(highlighted_i) = self.highlighted {
            assert!(
                highlighted_i >= num_skip_line as usize && highlighted_i < num_results as usize
            );
            let new_value = (highlighted_i as i32 + offset - num_skip_line)
                .rem_euclid(num_results - num_skip_line)
                + num_skip_line;
            self.highlighted = Some(new_value as usize);
        }
    }

    fn decrease_highlighted(&mut self) -> Result<()> {
        self.offset_highlighted(-1)
    }

    fn increase_highlighted(&mut self) -> Result<()> {
        self.offset_highlighted(1)
    }
}
