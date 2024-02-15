mod app;
mod display;
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
use ratatui::widgets::ListItem;

use clap::{Parser, ValueEnum};

use std::{
    error::Error,
    io::{self, Stdout},
};

use crate::app::App;
use crate::display::build_display;
use crate::display::DisplayMode;
use crate::job_handler::run_command;

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

    let mut app = App::new(cli);
    run(&mut terminal, &mut app)?;
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

fn run(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
) -> Result<(), Box<dyn Error>> {
    loop {
        let vec_line_display = app.make_ui();
        if event::poll(std::time::Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char(c) => app.send_char(c),
                    _ => panic!("Unhandeled command !"),
                }
            }
        }
        terminal.draw(|frame| {
            let list_items = vec_line_display
                .iter()
                .map(|line| ListItem::new(line.clone()))
                .collect::<Vec<_>>();
            let list = List::new(list_items)
                .block(
                    Block::default()
                        // .title("[q]uit [t]oggle_refresh [r]am [c]pu")
                        .title("[q]uit [t]oggle_refresh [l]ogs")
                        .title_position(Position::Bottom)
                        .borders(Borders::ALL),
                )
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
                .highlight_symbol(">>");
            frame.render_widget(list, frame.size());
        })?;
    }
    Ok(())
}
