use color_eyre::Result;
use phf::phf_map;
use ratatui::prelude::Color;
use tracing::info;

use crate::{jobs::job_handler, parser::RunMode, ui::Colorable};

static SACCT_MAP: phf::Map<&'static str, usize> = phf_map! {
    "JobID" => 0,
    "JobName" => 1,
    "Partition" => 2,
    "Account" => 3,
    "AllocCPUS" => 4,
    "State" => 5,
    "ExitCode" => 6,
    "SubmitLine" => 7,
    "WorkDir" => 8,
};

#[derive(Clone)]
pub struct JobFields {
    pub job_id: String,
    pub job_name: String,
    pub partition: String,
    pub account: String,
    pub alloc_cpus: String,
    pub state: JobState,
    pub exit_code: String,
    pub submit_line: String,
    pub workdir: String,
}

// from sacct doc
// BF BOOT_FAIL => Unknown
// CA CANCELLED => Cancelled
// CD COMPLETED => Completed
// DL DEADLINE => Timeout
// F FAILED => Failed
// NF NODE_FAIL => Unknown
// OOM OUT_OF_MEMORY => OutOfMemory
// PD PENDING => Pending
// PR PREEMPTED => Unknown
// R RUNNING => Running
// RQ REQUEUED => Unknown
// RS RESIZING => Unknown
// RV REVOKED => Unknown
// S SUSPENDED => Unknown
// TO TIMEOUT => Timeout

#[derive(Clone, PartialEq, Eq)]
pub enum JobState {
    Running,
    Completed,
    Cancelled(String),
    Pending,
    Failed,
    Header,
    Unknown(String),
}

impl JobState {
    fn from_str(s: &str) -> Self {
        let state_str = s.split_whitespace().next().unwrap_or("");
        match state_str {
            "COMPLETED" => JobState::Completed,
            "OUT_OF_MEMORY" | "CANCELLED" | "TIMEOUT" | "DEADLINE" => {
                JobState::Cancelled(s.to_string())
            }
            "PENDING" => JobState::Pending,
            "RUNNING" => JobState::Running,
            "FAILED" => JobState::Failed,
            "State" => JobState::Header,
            _ => JobState::Unknown(s.to_string()),
        }
    }
}

impl ToString for JobState {
    fn to_string(&self) -> String {
        match self {
            JobState::Completed => "COMPLETED".to_string(),
            JobState::Running => "RUNNING".to_string(),
            JobState::Failed => "FAILED".to_string(),
            JobState::Pending => "PENDING".to_string(),
            JobState::Header => "State".to_string(),
            JobState::Cancelled(s) => "CANCELLED".to_string() + "(" + s + ")",
            JobState::Unknown(s) => format!("Unknown({})", s),
        }
    }
}
impl Colorable for JobState {
    fn to_color(&self) -> Color {
        match self {
            JobState::Completed => Color::Green,
            JobState::Running => Color::LightGreen,
            JobState::Failed => Color::Red,
            JobState::Pending => Color::White,
            JobState::Header => Color::White,
            JobState::Cancelled(_) => Color::LightRed,
            JobState::Unknown(_) => Color::Cyan,
        }
    }
}

impl JobFields {
    pub fn from_slice(slice: Vec<String>) -> JobFields {
        assert_eq!(slice.len(), 9);
        JobFields {
            job_id: slice[0].clone(),
            job_name: slice[1].clone(),
            partition: slice[2].clone(),
            account: slice[3].clone(),
            alloc_cpus: slice[4].clone(),
            state: JobState::from_str(&slice[5]),
            exit_code: slice[6].clone(),
            submit_line: slice[7].clone(),
            workdir: slice[8].clone(),
        }
    }

    pub fn from_sacct_str(sacct_res: &str) -> Result<Vec<JobFields>> {
        //TODO(lhenches): use nom for parsing ?
        let line_vector: Vec<Vec<String>> = sacct_res
            .trim_end_matches('\n')
            .split('\n')
            .map(|line| {
                let raw_fields = line.split('|').map(|f| f.to_string()).collect::<Vec<_>>();
                assert_eq!(raw_fields.len(), SACCT_MAP.len());
                raw_fields
            })
            .collect();

        let all_fields = &line_vector[0];
        assert_eq!(all_fields.len(), SACCT_MAP.len());
        let hash_index: Vec<_> = all_fields.iter().map(|field| SACCT_MAP[field]).collect();

        let fields_correct_order = line_vector
            .into_iter()
            .map(|line| {
                hash_index
                    .iter()
                    .map(|&i| line[i].clone())
                    .collect::<Vec<String>>()
            })
            .map(JobFields::from_slice)
            .collect();

        Ok(fields_correct_order)
    }

    fn format_str(s: &str) -> String {
        format!("{:<25}", s)
    }

    pub fn display_lines(&self) -> String {
        [
            Self::format_str(&self.job_id),
            Self::format_str(&self.job_name),
            Self::format_str(&self.partition),
            Self::format_str(&self.account),
            Self::format_str(&self.alloc_cpus),
            Self::format_str(&self.state.to_string()),
            Self::format_str(&self.exit_code),
            Self::format_str(&self.submit_line),
            Self::format_str(&self.workdir),
        ]
        .join(" ")
    }
}
pub fn fetch_logs(run_mode: RunMode, fields: &JobFields) -> Result<Vec<String>> {
    // try to get log file
    let find_result =
        job_handler::get_log_files_finished_job(run_mode, &fields.workdir, &fields.job_id)?;
    info!(find_result);
    // parse logs into multiple files
    let vec_logs = if !find_result.is_empty() {
        find_result
            .trim_end_matches('\n')
            .split('\n')
            .map(|s| s.to_string())
            .collect()
    } else {
        vec![]
    };

    Ok(vec_logs)
}
