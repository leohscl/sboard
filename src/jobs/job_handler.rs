use crate::app::App;
use crate::app::JobQueryInfo;
use crate::app::JobTime;
use crate::parser::RunMode;
use clap::ValueEnum;
use color_eyre::eyre::Result;
use std::process::Command;
use tracing::info;

use super::job_parser::JobFields;

// fn run_squeue(run_mode: RunMode) -> Result<String> {
//     let squeue_args = vec!["--me", "--format=%all"];
//     run_command(run_mode, "squeue", &squeue_args)
// }

fn run_sacct(run_mode: RunMode) -> Result<String> {
    let squeue_args = vec!["--format=JobID,JobName,Partition,Account,AllocCPUS,State,ExitCode,SubmitLine%50,WorkDir%100", "-P"];
    run_command(run_mode, "sacct", &squeue_args)
}

pub fn fetch_jobs(app: &App, job_info: JobQueryInfo) -> Result<Vec<JobFields>> {
    let run_mode = app.cli.run_mode;
    let sacct_res = run_sacct(run_mode)?;
    let mut all_job_fields = JobFields::from_sacct_str(&sacct_res)?;
    // remove fields with empty partition
    all_job_fields.retain(|job_fields| !job_fields.partition.is_empty());
    match job_info.time {
        JobTime::Current => {
            all_job_fields
                .retain(|job_fields| job_fields.state == "RUNNING" || job_fields.state == "State");
            Ok(all_job_fields)
        }
        JobTime::Past => {
            all_job_fields
                .retain(|job_fields| job_fields.state != "RUNNING" || job_fields.state == "State");
            Ok(all_job_fields)
        }
    }
}

// pub fn run_scontrol(run_mode: RunMode, id: &str) -> Result<String> {
//     let scontrol_args = vec!["show", "job", id];
//     run_command(run_mode, "scontrol", &scontrol_args)
// }

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

// #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
// pub enum OutputFormat {
//     Sacct,
//     Squeue,
// }
