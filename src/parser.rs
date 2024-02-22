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
    #[arg(short, long, value_enum, default_value_t = RunMode::Slurm)]
    pub run_mode: RunMode,
    #[arg(long)]
    pub refresh: bool,
    #[arg(short, long, value_enum, default_value_t = DisplayMode::Cpu)]
    pub display_mode: DisplayMode,
    #[arg(short, long, value_parser = clap::value_parser!(u16).range(10..), default_value_t = 30)]
    pub job_max_display: u16,
}
