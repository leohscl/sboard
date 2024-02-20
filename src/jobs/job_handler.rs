use crate::app::App;
use crate::app::JobDetails;
use crate::app::JobInfo;
use crate::app::JobTime;
use crate::parser::RunMode;
use clap::ValueEnum;
use color_eyre::eyre::Result;
use color_eyre::Report;
use std::collections::HashMap;
use std::process::Command;

use super::job_parser::JobFields;

// fn run_squeue(run_mode: RunMode) -> Result<String> {
//     let squeue_args = vec!["--me", "--format=%all"];
//     run_command(run_mode, "squeue", &squeue_args)
// }

fn run_sacct(run_mode: RunMode) -> Result<String> {
    let squeue_args = vec!["--format=JobID,JobName,Partition,Account,AllocCPUS,State,ExitCode,SubmitLine%50,WorkDir%100", "-P"];
    run_command(run_mode, "sacct", &squeue_args)
}

pub fn fetch_jobs(app: &App, job_info: JobInfo) -> Result<Vec<String>> {
    let run_mode = app.cli.run_mode;
    let sacct_res = run_sacct(run_mode)?;
    let all_job_fields = JobFields::from_sacct_str(&sacct_res)?;
    let all_job_results = all_job_fields
        .iter()
        .map(|job_fields| job_fields.display_lines())
        .collect();
    match job_info.time {
        //TODO(lhenches): filter on status
        JobTime::Past => Ok(all_job_results),
        JobTime::Current => Ok(all_job_results),
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
    let find_args = [workdir, "-name", job_id];
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

fn format_sacct(sacct_res: &str) -> Result<Vec<String>> {
    let mut line_vector: Vec<Vec<String>> = sacct_res
        .trim_end_matches('\n')
        .split('\n')
        .map(|line| {
            line.split_whitespace()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        })
        .collect();
    line_vector.remove(1);
    let num_cols = line_vector
        .iter()
        .next()
        .ok_or(Report::msg("no first line in sacct results"))?
        .len();
    line_vector.retain(|v| v.len() == num_cols);
    Ok(line_vector.iter().map(|l| l.join(" ")).collect())
}

fn rename_field(field_raw: &str) -> String {
    let mut rename_dict = HashMap::new();
    rename_dict.insert("NODELIST(REASON)", "NODE".to_string());
    match rename_dict.get(field_raw) {
        Some(new_field) => new_field.clone(),
        None => field_raw.to_string(),
    }
}

pub fn parse_job_id(job_line: &str) -> Result<String> {
    // println!(job_line);
    let results = job_line
        .split_whitespace()
        .next()
        .ok_or(Report::msg("split failed"))?;
    Ok(results.to_string())
}

fn parse_start(lines: &[&str], pattern_start: &str) -> Result<String> {
    let parsed = lines.iter().find_map(|line| {
        if line.starts_with(pattern_start) {
            let path = line.split(pattern_start).nth(1).unwrap();
            Some(path.to_string())
        } else {
            None
        }
    });
    parsed.ok_or(Report::msg(pattern_start.to_string() + " not found"))
}

pub fn parse_job_details(job_id: &str, job_info: &str) -> Result<JobDetails> {
    let lines: Vec<&str> = job_info.split('\n').map(|s| s.trim()).collect();
    let err_file = parse_start(&lines, "StdErr=")?;
    let log_file = parse_start(&lines, "StdOut=")?;
    let job_detail = JobDetails {
        job_id: job_id.to_string(),
        err_file,
        log_file,
    };
    Ok(job_detail)
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum DisplayMode {
    Cpu,
    Ram,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OutputFormat {
    Sacct,
    Squeue,
}
