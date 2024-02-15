use crate::RunMode;
use std::error::Error;
use std::process::Command;

pub fn run_command(run_mode: &RunMode, args: &Vec<&str>) -> Result<String, Box<dyn Error>> {
    let output_jobs = match run_mode {
        RunMode::Slurm => Command::new("squeue").args(args).output()?,
        RunMode::FromFile => Command::new("/bin/cat")
            .arg("test_data/10_random_users.txt")
            .output()?,
        RunMode::Ssh => Command::new("ssh")
            .arg("maestro")
            .arg("squeue")
            .args(args)
            .output()?,
    };
    let out_txt = String::from_utf8(output_jobs.stdout)?;
    Ok(out_txt)
}
