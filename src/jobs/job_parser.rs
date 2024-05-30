use chrono::NaiveDateTime;
use chrono::NaiveTime;
use chrono::Timelike;
use color_eyre::Report;
use color_eyre::Result;
use phf::phf_map;
use ratatui::prelude::Color;
use std::default::Default;
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
    "Submit" => 9,
    "ReqMem" => 10,
    "MaxRSS" => 11,
    "ElapsedRaw" => 12,
    "TimelimitRaw" => 13,
    "TotalCPU" => 14,
};

#[derive(Clone, Debug)]
pub struct JobFields {
    pub job_id: String,
    pub job_name: String,
    pub partition: String,
    pub account: String,
    pub alloc_cpus: NumberOrCol,
    pub state: JobState,
    pub exit_code: String,
    pub submit_line: String,
    pub workdir: String,
    pub submit: Option<NaiveDateTime>,
    pub reqmem: NumberOrCol,
    pub maxrss: NumberOrCol,
    pub elapsed: NumberOrCol,
    pub time_limit: NumberOrCol,
    pub cpu_time_raw: NumberOrCol,
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

#[derive(Clone, PartialEq, Eq, Debug)]
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

#[derive(Clone, Debug)]
pub enum Unit {
    Gigabytes,
    Kilobytes,
}

impl Unit {
    fn get_result_kilobytes(&self, number: usize) -> usize {
        let multiplicator = match self {
            Unit::Kilobytes => 1,
            Unit::Gigabytes => 1 << 20,
        };
        multiplicator * number
    }
}

#[derive(Clone, Debug)]
pub enum NumberOrCol {
    Value(usize),
    Col(String),
}

impl NumberOrCol {
    pub fn as_string(&self) -> String {
        match self {
            NumberOrCol::Value(usize) => usize.to_string(),
            NumberOrCol::Col(s) => s.clone(),
        }
    }
    pub fn take(self) -> Option<usize> {
        match self {
            Self::Value(t) => Some(t),
            Self::Col(_) => None,
        }
    }
    pub fn from_str(s: &str) -> Self {
        if s.is_empty() {
            return NumberOrCol::Value(usize::default());
        }
        if let Ok(v) = s.parse() {
            return Self::Value(Unit::Kilobytes.get_result_kilobytes(v));
        };
        let (num, last_char) = s.split_at(s.len() - 1);
        let unit = match last_char {
            "K" => Unit::Kilobytes,
            "G" => Unit::Gigabytes,
            _ => return Self::Col(s.to_string()),
        };
        match num.parse() {
            Ok(v) => Self::Value(unit.get_result_kilobytes(v)),
            Err(_) => Self::Col(s.to_string()),
        }
    }
}

fn parse_elapsed_format_secs(s: &str) -> Result<usize> {
    // TODO(lhenches): handle optional [days-]
    let usable_string = if s.contains('.') {
        "00:".to_string() + s.split('.').next().unwrap()
    } else {
        s.to_string()
    };
    info!("usable_string: {}", usable_string);
    // TODO(lhenches): refactor !
    let res = NaiveTime::parse_from_str(&usable_string, "%H:%M:%S");
    info!("res: {:?}", res);
    if let Ok(t) = res {
        return Ok(t.num_seconds_from_midnight() as usize);
    };
    Err(Report::msg("Parsing error"))
}

impl JobFields {
    pub fn from_slice(slice: Vec<String>) -> Result<JobFields> {
        // assert_eq!(slice.len(), 10);
        let opt_submit_date = match slice[9].as_str() {
            "Submit" => None,
            s => Some(NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")?),
        };
        let alloc_cpus = NumberOrCol::from_str(&slice[4]);
        info!("alloc_cpus: {:?}", alloc_cpus);
        let reqmem = NumberOrCol::from_str(&slice[10]);
        let maxrss = NumberOrCol::from_str(&slice[11]);
        let elapsed = NumberOrCol::from_str(&slice[12]);
        let time_limit = NumberOrCol::from_str(&slice[13]);
        let cpu_time_raw = match slice[14].as_str() {
            "TotalCPU" => NumberOrCol::Col("TotalCPU".to_string()),
            s => {
                let elapsed_cpu = parse_elapsed_format_secs(s).unwrap();
                NumberOrCol::Value(elapsed_cpu)
            }
        };
        // let total_cpu = NumberOrCol::from_str(&slice[14]);
        // info!("maxrss: {}, reqmem: {}", maxrss, reqmem);
        // let memory_eff = (maxrss as f64 / reqmem as f64) * 100f64;
        // let formated_mem_eff = format!("{:.1$}", memory_eff, 1);
        let job_fields = JobFields {
            job_id: slice[0].clone(),
            job_name: slice[1].clone(),
            partition: slice[2].clone(),
            account: slice[3].clone(),
            alloc_cpus,
            state: JobState::from_str(&slice[5]),
            exit_code: slice[6].clone(),
            submit_line: slice[7].clone(),
            workdir: slice[8].clone(),
            submit: opt_submit_date,
            reqmem,
            maxrss,
            elapsed,
            time_limit,
            cpu_time_raw,
        };
        Ok(job_fields)
    }

    pub fn get_mem_eff(&self) -> String {
        match (self.reqmem.clone().take(), self.maxrss.clone().take()) {
            (Some(req), Some(max)) => {
                let memory_eff = (max as f64 / req as f64) * 100f64;
                format!("{:.1$}%", memory_eff, 1)
            }
            (_, _) => "MemEff".to_string(),
        }
    }

    pub fn get_time_eff(&self) -> String {
        match (self.elapsed.clone().take(), self.time_limit.clone().take()) {
            (Some(elap), Some(limit)) => {
                let time_eff = (elap as f64 / limit as f64) * 100f64;
                format!("{:.1$}%", time_eff, 1)
            }
            (_, _) => "TimeEff".to_string(),
        }
    }

    pub fn get_cpu_eff(&self) -> String {
        match (
            self.alloc_cpus.clone().take(),
            self.elapsed.clone().take(),
            self.cpu_time_raw.clone().take(),
        ) {
            (Some(num_cpu), Some(elapsed), Some(total)) => {
                let max = elapsed * num_cpu;
                let cpu_eff = (total as f64 / max as f64) * 100f64;
                format!("{:.1$}%", cpu_eff, 1)
            }
            _ => "CPUEff".to_string(),
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
        let hash_index: Vec<_> = all_fields
            .iter()
            .inspect(|f| {
                info!(f);
            })
            .map(|field| SACCT_MAP[field])
            .collect();

        let fields_correct_order = line_vector
            .into_iter()
            .map(|line| {
                hash_index
                    .iter()
                    .map(|&i| line[i].clone())
                    .collect::<Vec<String>>()
            })
            .map(JobFields::from_slice)
            .collect::<Result<Vec<_>>>()?;

        Ok(fields_correct_order)
    }

    fn format_str(s: &str, num_c: usize) -> String {
        format!(
            "{:width$}",
            format!("{:.width$}", s, width = num_c),
            width = num_c
        )
    }

    pub fn display_lines(&self, efficiency_display: bool) -> String {
        let submit_fmt = if let Some(ref date) = self.submit {
            date.format("%Y-%m-%d %H:%M:%S").to_string()
        } else {
            "Submit".to_string()
        };
        let mut vec_strings_display = vec![
            Self::format_str(&self.job_id, 15),
            Self::format_str(&self.job_name, 20),
            Self::format_str(&self.partition, 14),
            Self::format_str(&self.alloc_cpus.as_string(), 14),
            Self::format_str(&self.state.to_string(), 35),
        ];
        if efficiency_display {
            vec_strings_display.extend([
                Self::format_str(&self.get_time_eff(), 10),
                Self::format_str(&self.get_cpu_eff(), 10),
                Self::format_str(&self.get_mem_eff(), 10),
            ]);
        } else {
            vec_strings_display.extend([
                Self::format_str(&self.exit_code, 10),
                Self::format_str(&self.submit_line, 25),
                Self::format_str(&submit_fmt, 20),
            ]);
        }
        vec_strings_display.join(" ")
    }
}
pub fn fetch_logs(run_mode: RunMode, fields: &JobFields) -> Result<Vec<String>> {
    // try to get log file
    let find_result =
        job_handler::get_log_files_finished_job(run_mode, &fields.workdir, &fields.job_id)?;
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
