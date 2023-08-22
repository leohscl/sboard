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
use ratatui::widgets::ListItem;

use clap::{Parser, ValueEnum};

use std::{
    error::Error,
    io::{self, Stdout},
};

use std::process::Command;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum RunMode {
    Slurm,
    FromFile,
    Ssh,
}

#[derive(Parser)]
struct Cli {
    #[arg(value_enum)]
    run_mode: RunMode,
    #[arg(long)]
    refresh: bool,
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
    let mut current_display = run_command(cli.run_mode)?;
    let mut refresh = cli.refresh;
    Ok(loop {
        if refresh {
            current_display = run_command(cli.run_mode)?
        }
        let jobs: Vec<_> = current_display
            .trim_end_matches("\n")
            .split("\n")
            .map(|line| ListItem::new(line))
            .collect();
        terminal.draw(|frame| {
            let list = List::new(jobs)
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

fn run_command(run_mode: RunMode) -> Result<String, Box<dyn Error>> {
    let output_jobs = match run_mode {
        RunMode::Slurm => Command::new("squeue").arg("--me").output()?,
        RunMode::FromFile => Command::new("/bin/cat")
            .arg("test_data/10_random_users.txt")
            .output()?,
        RunMode::Ssh => Command::new("ssh")
            .arg("maestro")
            .arg("squeue")
            .arg("--me")
            .output()?,
    };
    let out_txt = String::from_utf8(output_jobs.stdout)?;
    Ok(out_txt)
}
