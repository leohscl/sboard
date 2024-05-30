mod app;
mod editor;
mod job_query_info;
mod jobs;
mod parser;
mod ui;

use crate::app::App;
use better_panic::Settings;
use clap::Parser;
use color_eyre::eyre::Result;
use crossterm::event;
use crossterm::event::Event;
use crossterm::execute;
use crossterm::terminal::*;
use parser::Cli;
use ratatui::prelude::*;
use std::io::{self, Stdout};
use tracing::info;
use ui::ui;

pub fn initialize_panic_handler() {
    std::panic::set_hook(Box::new(|panic_info| {
        crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen).unwrap();
        crossterm::terminal::disable_raw_mode().unwrap();
        Settings::auto()
            .most_recent_first(false)
            .lineno_suffix(true)
            .create_panic_handler()(panic_info);
    }));
}

fn main() -> Result<()> {
    color_eyre::install()?;
    initialize_panic_handler();
    setup_logging();
    let cli = Cli::parse();
    let mut terminal = setup_terminal()?;
    let mut app = App::new(cli);
    let run_result = run(&mut terminal, &mut app);
    restore_terminal(&mut terminal)?;
    run_result?;
    Ok(())
}

#[cfg(debug_assertions)]
fn setup_logging() {
    let logfile = tracing_appender::rolling::never("logs", "log.txt");
    let file_subscriber = tracing_subscriber::fmt().with_writer(logfile).finish();
    tracing::subscriber::set_global_default(file_subscriber)
        .expect("setting file subscriber failed");
}

#[cfg(not(debug_assertions))]
fn setup_logging() {
    // Do nothing in release mode
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

fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> Result<()> {
    loop {
        app.fetch_jobs()?;
        terminal.draw(|frame| {
            ui(frame, app);
        })?;
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                // info!(?key);
                let should_quit = app.send_keycode(key.code)?;
                if should_quit {
                    break;
                }
            }
        }
    }
    Ok(())
}
