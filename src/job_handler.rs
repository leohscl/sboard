use crate::Cli;
use crate::RunMode;
use clap::ValueEnum;
use ratatui::widgets::ListItem;
use std::process::Command;

use std::collections::HashMap;
use std::error::Error;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum DisplayMode {
    CPU,
    RAM,
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
            DisplayMode::CPU => vec!["NODES"],
            DisplayMode::RAM => vec![""],
        };
        let mut fields = default_fields.to_vec();
        fields.append(&mut specific_fields);
        fields.into_iter().map(|str| str.to_string()).collect()
    }
}

fn rename_field(field_raw: &str) -> String {
    let mut rename_dict = HashMap::new();
    rename_dict.insert("NODELIST(REASON)", "NODE".to_string());
    match rename_dict.get(field_raw) {
        Some(new_field) => new_field.clone(),
        None => field_raw.to_string(),
    }
}

pub fn build_display<'a>(
    job_raw_output: &'a str,
    cli: &Cli,
) -> Result<Vec<ListItem<'a>>, Box<dyn Error>> {
    let line_vector: Vec<Vec<String>> = job_raw_output
        .trim_end_matches("\n")
        .split("\n")
        .enumerate()
        .map(|(index, line)| {
            line.split("|")
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
    let mut fields_keep = cli.display_mode.get_fields();
    let all_fields = &line_vector[0];
    let indicies: Vec<_> = all_fields
        .into_iter()
        .enumerate()
        .filter_map(|(index, field)| {
            if fields_keep.contains(&field) {
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
            let line = line_fields
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
                .join(" ");
            ListItem::new(line)
        })
        .collect();
    Ok(job_lines)
}

pub fn run_command(run_mode: RunMode) -> Result<String, Box<dyn Error>> {
    let squeue_args = ["--me", "--format=%all"];
    let output_jobs = match run_mode {
        RunMode::Slurm => Command::new("squeue").args(squeue_args).output()?,
        RunMode::FromFile => Command::new("/bin/cat")
            .arg("test_data/10_random_users.txt")
            .output()?,
        RunMode::Ssh => Command::new("ssh")
            .arg("maestro")
            .arg("squeue")
            .args(squeue_args)
            .output()?,
    };
    let out_txt = String::from_utf8(output_jobs.stdout)?;
    Ok(out_txt)
}
