use crate::app::App;
use crate::app::JobDetails;
use crate::parser::RunMode;
use clap::ValueEnum;
use color_eyre::eyre::Result;
use color_eyre::Report;
use std::collections::HashMap;
use std::process::Command;

pub fn run_squeue(run_mode: RunMode) -> Result<String> {
    let squeue_args = vec!["--me", "--format=%all"];
    run_command(run_mode, "squeue", &squeue_args)
}

pub fn run_scontrol(run_mode: RunMode, id: &str) -> Result<String> {
    let scontrol_args = vec!["show", "job", id];
    run_command(run_mode, "scontrol", &scontrol_args)
}

pub fn read_file(run_mode: RunMode, path: &str) -> Result<String> {
    let cat_args = vec![path];
    run_command(run_mode, "cat", &cat_args)
}

fn run_command(run_mode: RunMode, cmd: &str, command_args: &Vec<&str>) -> Result<String> {
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

pub fn build_display(job_raw_output: String, app: &App) -> Result<Vec<String>> {
    let display_mode = app.get_display();

    let line_vector: Vec<Vec<String>> = job_raw_output
        .trim_end_matches('\n')
        .split('\n')
        .enumerate()
        .map(|(index, line)| {
            line.split('|')
                .map(|str| {
                    if index == 0 {
                        rename_field(str)
                    } else {
                        str.to_string()
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect();
    let mut fields_keep = display_mode.get_fields();
    let all_fields = &line_vector[0];
    let indicies: Vec<_> = all_fields
        .iter()
        .enumerate()
        .filter_map(|(index, field)| {
            if fields_keep.contains(field) {
                fields_keep.retain(|f| f != field);
                Some(index)
            } else {
                None
            }
        })
        .collect();
    let job_lines = line_vector
        .into_iter()
        .map(|line_fields| {
            line_fields
                .into_iter()
                .enumerate()
                .filter_map(|(index, field)| {
                    if indicies.contains(&index) {
                        let field_fmt = format!("{:12}", field);
                        Some(field_fmt)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect();
    Ok(job_lines)
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

impl DisplayMode {
    fn get_fields(&self) -> Vec<String> {
        let default_fields = [
            "JOBID",
            "PARTITION",
            "NAME",
            "USER",
            "STATE",
            "TIME",
            "TIME_LIMI",
            "NODE",
        ];
        let mut specific_fields = match self {
            DisplayMode::Cpu => vec!["NODES"],
            DisplayMode::Ram => vec![""],
        };
        let mut fields = default_fields.to_vec();
        fields.append(&mut specific_fields);
        fields.into_iter().map(|str| str.to_string()).collect()
    }
}
