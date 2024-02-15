mod app;
mod display;
mod job_handler;
mod parser;

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
use ratatui::widgets::block::Position;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::List;
use ratatui::widgets::ListItem;
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

fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> Result<()> {
    loop {
        let vec_line_display = app.make_ui();

        terminal.draw(|frame| {
            let list_items = vec_line_display
                .iter()
                .map(|line| ListItem::new(line.clone()))
                .collect::<Vec<_>>();
            let list = List::new(list_items)
                .block(
                    Block::default()
                        .title("[q]uit [t]oggle_refresh [l]ogs")
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
                    KeyCode::Char(c) => app.send_char(c),
                    _ => panic!("Unhandeled command !"),
                }
            }
        }
    }
    Ok(())
}
