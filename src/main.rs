mod app;
mod job_handler;
mod parser;
mod ui;

use crate::app::App;
use clap::Parser;
use color_eyre::eyre::Report;
use color_eyre::eyre::Result;
use crossterm::event;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::execute;
use crossterm::terminal::*;
use parser::Cli;
use ratatui::prelude::*;
use std::io::{self, Stdout};
use ui::ui;

fn main() -> Result<()> {
    color_eyre::install()?;
    let logfile = tracing_appender::rolling::never("/home/leo/project/sboard/logs", "log.txt");
    let file_subscriber = tracing_subscriber::fmt().with_writer(logfile).finish();
    tracing::subscriber::set_global_default(file_subscriber)
        .expect("setting file subscriber failed");
    let cli = Cli::parse();
    let mut terminal = setup_terminal()?;
    let mut app = App::new(cli);
    let run_result = run(&mut terminal, &mut app);
    restore_terminal(&mut terminal)?;
    run_result?;
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

fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> Result<()> {
    loop {
        app.fetch_jobs()?;
        terminal.draw(|frame| ui(frame, app))?;
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        if app.send_quit() {
                            break;
                        }
                    }
                    KeyCode::Char(c) => app.send_char(c)?,
                    KeyCode::Enter => app.fetch_job_info()?,
                    _ => return Err(Report::msg("Unhandeled key press !")),
                }
            }
        }
    }
    Ok(())
}
