use crate::jobs::job_handler::DisplayMode;
use clap::Parser;
#[derive(Parser)]
pub struct Cli {
    #[arg(short, long)]
    pub from_file: Option<String>,
    #[arg(long)]
    pub refresh: bool,
    #[arg(short, long, value_enum, default_value_t = DisplayMode::Cpu)]
    pub display_mode: DisplayMode,
    #[arg(short, long, value_parser = clap::value_parser!(u16).range(10..), default_value_t = 30)]
    pub job_max_display: u16,
}
