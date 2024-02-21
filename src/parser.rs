use crate::jobs::job_handler::DisplayMode;
use clap::{Parser, ValueEnum};
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum RunMode {
    Slurm,
    FromFile,
    Ssh,
}
#[derive(Parser)]
pub struct Cli {
    #[arg(value_enum, default_value_t = RunMode::Slurm)]
    pub run_mode: RunMode,
    #[arg(long)]
    pub refresh: bool,
    #[arg(value_enum, default_value_t = DisplayMode::Cpu)]
    pub display_mode: DisplayMode,
}
