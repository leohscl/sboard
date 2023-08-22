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
use std::{
    error::Error,
    io::{self, Stdout},
};

use std::process::Command;

fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = setup_terminal()?;
    run(&mut terminal)?;
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

fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), Box<dyn Error>> {
    Ok(loop {
        let output_jobs = Command::new("/bin/cat")
            .arg("test_data/10_random_users.txt")
            .output()
            .expect("failed to run command");
        let out_txt = String::from_utf8(output_jobs.stdout)
            .expect("Unexpected error in string creation, from_utf8");
        let jobs: Vec<_> = out_txt
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
                if KeyCode::Char('q') == key.code {
                    break;
                }
            }
        }
    })
}
