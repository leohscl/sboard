use crate::app::App;
use ratatui::prelude::*;
use ratatui::widgets::block::Position;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::List;
use ratatui::widgets::ListItem;
use ratatui::Frame;

pub fn ui(frame: &mut Frame, app: &App) {
    if let Some(results) = app.results.clone() {
        let list_items = results
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
    }
}
