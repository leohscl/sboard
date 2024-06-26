use crate::job_query_info::JobQueryInfo;
use crate::job_query_info::JobTime;
use crate::jobs::job_handler;
use crate::jobs::job_parser::JobFields;
use crate::Cli;
use crate::{editor::Editor, jobs::job_parser};
use chrono::NaiveDateTime;
use color_eyre::eyre::{Ok, Report, Result};
use core::panic;
use crossterm::event::KeyCode;
use tracing::info;

pub enum DisplayState<'a> {
    Empty,
    Jobs(JobQueryInfo),
    Logs(Vec<String>),
    Editor(Editor<'a>),
}

#[derive(Clone)]
pub struct MyPopup {
    pub popup_text: String,
}

#[derive(Clone, Copy)]
pub enum FetchTime {
    Today,
    ThreeDaysAgo,
    AWeekAgo,
    SpecificWindow {
        start: NaiveDateTime,
        end: NaiveDateTime,
    },
}

impl FetchTime {
    fn older(&self) -> FetchTime {
        match self {
            Today => ThreeDaysAgo,
            ThreeDaysAgo => AWeekAgo,
            AWeekAgo => AWeekAgo,
            _ => *self,
        }
    }
    fn newer(&self) -> FetchTime {
        match self {
            Today => Today,
            ThreeDaysAgo => Today,
            AWeekAgo => ThreeDaysAgo,
            _ => *self,
        }
    }
}

use FetchTime::*;

pub struct App<'a> {
    pub cli: Cli,
    pub display_state: DisplayState<'a>,
    pub cached_display: Option<DisplayState<'a>>,
    pub highlighted: Option<usize>,
    pub cached_highlight: Option<usize>,
    pub popup: Option<MyPopup>,
    pub fetch_time: FetchTime,
}

impl<'a> App<'a> {
    pub fn new(cli: Cli) -> App<'a> {
        App {
            cli,
            cached_display: None,
            highlighted: None,
            cached_highlight: None,
            popup: None,
            display_state: DisplayState::Empty,
            fetch_time: Today,
        }
    }

    fn send_enter(&mut self) -> Result<bool> {
        match self.display_state {
            DisplayState::Empty => Ok(false),
            DisplayState::Jobs(_) => self.send_char('l'),
            DisplayState::Logs(_) => self.send_char('v'),
            DisplayState::Editor(_) => Ok(false),
        }
    }

    pub fn fetch_jobs(&mut self) -> Result<()> {
        // Only fetch results if needed
        let job_info = match self.display_state {
            DisplayState::Jobs(ref mut j_info) => {
                if j_info.refresh | j_info.changed {
                    j_info.changed = false;
                    j_info.clone()
                } else {
                    return Ok(());
                }
            }
            DisplayState::Empty => JobQueryInfo::default(self),
            _ => return Ok(()),
        };
        let job_results = job_handler::fetch_jobs(self, job_info)?;
        self.highlighted = if job_results.len() >= 2 {
            Some(1)
        } else {
            None
        };
        self.update_job_display(job_results);
        Ok(())
    }

    fn update_job_display(&mut self, new_results: Vec<JobFields>) {
        if let DisplayState::Jobs(ref mut job_info) = self.display_state {
            job_info.job_list = new_results;
            job_info.make_display()
        } else {
            let mut jqi = JobQueryInfo::from_result(new_results, self);
            jqi.make_display();
            self.display_state = DisplayState::Jobs(jqi);
        }
    }
    pub fn send_keycode(&mut self, keycode: KeyCode) -> Result<bool> {
        if self.popup.is_some() {
            self.popup = None;
            Ok(false)
        } else {
            match keycode {
                KeyCode::Char(c) => self.send_char(c),
                KeyCode::Enter => self.send_enter(),
                KeyCode::Down => {
                    self.increase_highlighted()?;
                    Ok(false)
                }
                KeyCode::Up => {
                    self.decrease_highlighted()?;
                    Ok(false)
                }
                _ => Ok(false),
            }
        }
    }

    fn get_highlighted_i(&self) -> Result<usize> {
        self.highlighted.ok_or(Report::msg("No highlights"))
    }

    fn send_quit(&mut self) -> bool {
        match self.display_state {
            DisplayState::Logs(_) | DisplayState::Editor(_) => {
                if let Some(cached) = self.cached_display.take() {
                    self.highlighted = self.cached_highlight;
                    self.cached_highlight = None;
                    self.display_state = cached;
                    self.cached_display = None;
                } else {
                    self.display_state = DisplayState::Empty;
                }
                false
            }
            DisplayState::Jobs(_) | DisplayState::Empty => true,
        }
    }

    fn offset_highlighted(&mut self, offset: i32) -> Result<()> {
        match self.display_state {
            DisplayState::Jobs(ref job_info) => {
                let num_skip_line = 1;
                let num_results = job_info.job_display.len();
                self.offset_highlighted_with_params(offset, num_results, num_skip_line);
            }
            DisplayState::Logs(ref strings) => {
                let num_results = strings.len();
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
        num_results: usize,
        num_skip_line: i32,
    ) {
        if let Some(highlighted_i) = self.highlighted {
            assert!(highlighted_i >= num_skip_line as usize && highlighted_i < num_results);
            let new_value = (highlighted_i as i32 + offset - num_skip_line)
                .rem_euclid(num_results as i32 - num_skip_line)
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

pub static DESCRIPTION_JOB: &str =
    "[q]uit [t]oggle_refresh [l]ogs [f]inished [r]unning [a]ll [s]eff [o]lder [n]ewer";
pub static DESCRIPTION_LOG: &str = "[q]uit [v]iew";

impl<'a> App<'a> {
    fn send_char(&mut self, c_sent: char) -> Result<bool> {
        let res_highlighted_i = self.get_highlighted_i();
        match (c_sent, &mut self.display_state) {
            ('q', _) => return Ok(self.send_quit()),
            (_, DisplayState::Editor(ref mut editor)) => editor.send_char(c_sent),
            (_, DisplayState::Empty) => (),
            ('l', DisplayState::Jobs(ref mut job_info)) => {
                let job_fields = &job_info.job_display[res_highlighted_i?];
                let logs = job_parser::fetch_logs(self.cli.run_mode, job_fields)?;
                if logs.is_empty() {
                    self.popup = Some(MyPopup {
                        popup_text: "No log file found.".to_string(),
                    })
                } else {
                    self.highlighted = Some(0);
                    self.display_state = DisplayState::Logs(logs);
                }
            }
            ('o', DisplayState::Jobs(ref mut job_info)) => {
                self.fetch_time = self.fetch_time.older();
                job_info.changed = true;
            }
            ('n', DisplayState::Jobs(ref mut job_info)) => {
                self.fetch_time = self.fetch_time.newer();
                job_info.changed = true;
            }
            ('t', DisplayState::Jobs(ref mut job_info)) => job_info.refresh = !job_info.refresh,
            ('f', DisplayState::Jobs(ref mut job_info)) => {
                job_info.time = JobTime::Finished;
                job_info.changed = true;
            }
            ('r', DisplayState::Jobs(ref mut job_info)) => {
                job_info.time = JobTime::Running;
                job_info.changed = true;
            }
            ('s', DisplayState::Jobs(ref mut job_info)) => {
                job_info.efficiency_display = true;
            }
            ('a', DisplayState::Jobs(ref mut job_info)) => {
                job_info.time = JobTime::All;
                job_info.changed = true;
            }
            ('v', DisplayState::Logs(logs)) => {
                self.cached_display = Some(DisplayState::Logs(logs.clone()));
                self.cached_highlight = self.highlighted;
                let logs = job_handler::read_file(self.cli.run_mode, &logs[res_highlighted_i?])?;
                self.display_state = DisplayState::Editor(Editor::new(&logs));
            }
            // ('j' | 'k', DisplayState::Report(_)) => {}
            ('j', _) => self.increase_highlighted()?,
            ('k', _) => self.decrease_highlighted()?,
            _ => (),
        }
        Ok(false)
    }
}
