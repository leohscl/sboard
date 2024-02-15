mod app;
mod job_handler;
mod parser;
mod ui;

use crate::app::App;
use clap::Parser;
use color_eyre::eyre::Result;
use crossterm::event;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::execute;
use crossterm::terminal::*;
use parser::Cli;
use ratatui::prelude::*;
use std::io::{self, Stdout};

fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();
    let mut terminal = setup_terminal()?;
    let mut app = App::new(cli);
    run(&mut terminal, &mut app)?;
    restore_terminal(&mut terminal)?;
    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen,)?;
    Ok(terminal.show_cursor()?)
}
use ui::ui;

fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> Result<()> {
    loop {
        app.fetch_results();
        terminal.draw(|frame| ui(frame, app))?;
        if event::poll(std::time::Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char(c) => app.send_char(c),
                    _ => panic!("Unhandeled command !"),
                }
            }
        }
    }
    Ok(())
}
