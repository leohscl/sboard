mod job_handler;
use crossterm::event;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::execute;
use crossterm::terminal::*;
use ratatui::prelude::*;
use ratatui::widgets::block::Position;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::List;

use clap::{Parser, ValueEnum};

use std::{
    error::Error,
    io::{self, Stdout},
};

use crate::job_handler::build_display;
use crate::job_handler::run_command;
use crate::job_handler::DisplayMode;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum RunMode {
    Slurm,
    FromFile,
    Ssh,
}

#[derive(Parser)]
pub struct Cli {
    #[arg(value_enum, default_value_t = RunMode::Ssh)]
    run_mode: RunMode,
    #[arg(long)]
    refresh: bool,
    #[arg(value_enum, default_value_t = DisplayMode::CPU)]
    display_mode: DisplayMode,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let mut terminal = setup_terminal()?;
    run(&mut terminal, &cli)?;
    restore_terminal(&mut terminal)?;
    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>, Box<dyn Error>> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

fn restore_terminal(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen,)?;
    Ok(terminal.show_cursor()?)
}

fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>, cli: &Cli) -> Result<(), Box<dyn Error>> {
    let mut current_job_info = run_command(cli.run_mode)?;
    let mut vec_line_display = build_display(&current_job_info, cli)?;
    let mut refresh = cli.refresh;
    Ok(loop {
        if refresh {
            current_job_info = run_command(cli.run_mode)?;
            vec_line_display = build_display(&current_job_info, cli)?;
        }
        terminal.draw(|frame| {
            let list = List::new(vec_line_display.clone())
                .block(
                    Block::default()
                        .title("[q]uit [t]oggle_refresh [r]am [c]pu")
                        .title_position(Position::Bottom)
                        .borders(Borders::ALL),
                )
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
                .highlight_symbol(">>");
            frame.render_widget(list, frame.size());
        })?;
        if event::poll(std::time::Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('t') => refresh = !refresh,
                    _ => (),
                }
            }
        }
    })
}
