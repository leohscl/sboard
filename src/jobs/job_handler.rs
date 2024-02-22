use crate::app::App;
use crate::app::JobQueryInfo;
use crate::app::JobTime;
use crate::parser::RunMode;
use clap::ValueEnum;
use color_eyre::eyre::Result;
use std::process::Command;
use tracing::info;

use super::job_parser::JobFields;
use super::job_parser::JobState;

fn run_sacct(run_mode: RunMode, hours_before_now: u16) -> Result<String> {
    let fmt_time = format!("now-{}hours", hours_before_now);
    let sacct_args = vec!["--format=JobID,JobName,Partition,Account,AllocCPUS,State,ExitCode,SubmitLine%50,WorkDir%100", "-P", "-S", &fmt_time];
    run_command(run_mode, "sacct", &sacct_args)
}

pub fn fetch_jobs(app: &App, job_info: JobQueryInfo) -> Result<Vec<JobFields>> {
    let cli = &app.cli;
    let sacct_res = run_sacct(cli.run_mode, cli.hours_before_now)?;
    let mut all_job_fields = JobFields::from_sacct_str(&sacct_res)?;
    // remove fields with empty partition
    all_job_fields.retain(|job_fields| !job_fields.partition.is_empty());
    match job_info.time {
        JobTime::Running => {
            all_job_fields.retain(|job_fields| match job_fields.state {
                JobState::Running | JobState::Header => true,
                _ => false,
            });
        }
        JobTime::Finished => {
            all_job_fields.retain(|job_fields| match job_fields.state {
                JobState::Running => false,
                _ => true,
            });
        }
        JobTime::All => (),
    }
    let job_fields_capped = all_job_fields
        .into_iter()
        .take(app.cli.job_max_display as usize)
        .collect();
    Ok(job_fields_capped)
}

pub fn get_log_files_finished_job(
    run_mode: RunMode,
    workdir: &str,
    job_id: &str,
) -> Result<String> {
    let regex = String::from("*") + job_id + "*";
    let find_args = [workdir, "-name", &regex];
    info!(?find_args);
    run_command(run_mode, "find", &find_args)
}

pub fn read_file(run_mode: RunMode, path: &str) -> Result<String> {
    let cat_args = vec![path];
    run_command(run_mode, "cat", &cat_args)
}

fn run_command(run_mode: RunMode, cmd: &str, command_args: &[&str]) -> Result<String> {
    let output_jobs = match run_mode {
        RunMode::Slurm => Command::new(cmd).args(command_args).output()?,
        RunMode::FromFile => Command::new("/bin/cat")
            .arg("test_data/10_random_users.txt")
            .output()?,
        RunMode::Ssh => Command::new("ssh")
            .arg("maestro")
            .arg(cmd)
            .args(command_args)
            .output()?,
    };
    let out_txt = String::from_utf8(output_jobs.stdout)?;
    Ok(out_txt)
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum DisplayMode {
    Cpu,
    Ram,
}
