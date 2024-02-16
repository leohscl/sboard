use crate::app::App;
use crate::app::DisplayState;
use ratatui::prelude::*;
use ratatui::widgets::block::Position;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::List;
use ratatui::widgets::ListItem;
use ratatui::Frame;

pub fn ui(frame: &mut Frame, app: &App) {
    match &app.display_state {
        DisplayState::Jobs => {
            if let Some(results) = app.jobs.clone() {
                let list_items = results
                    .iter()
                    .enumerate()
                    .map(|(i, line)| {
                        if let Some(highlighted_i) = app.highlighted {
                            if highlighted_i == i {
                                build_list_item(line, LineType::Highlighted)
                            } else {
                                build_list_item(line, LineType::Normal)
                            }
                        } else {
                            build_list_item(line, LineType::Normal)
                        }
                    })
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
        DisplayState::Details(details) => {
            let strings = [details.err_file.clone(), details.log_file.clone()];
            let list_items: Vec<_> = strings
                .iter()
                .map(|file| build_list_item(file, LineType::Normal))
                .collect();
            let list = List::new(list_items)
                .block(
                    Block::default()
                        .title("[q]uit [v]iew")
                        .title_position(Position::Bottom)
                        .borders(Borders::ALL),
                )
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
                .highlight_symbol(">>");
            frame.render_widget(list, frame.size());
        }
    }
}

enum LineType {
    Normal,
    Highlighted,
}

fn build_list_item(line: &str, line_type: LineType) -> ListItem {
    match line_type {
        LineType::Normal => {
            ListItem::new(line).style(Style::default().fg(Color::White).bg(Color::Black))
        }
        LineType::Highlighted => {
            ListItem::new(line).style(Style::default().fg(Color::Yellow).bg(Color::Black))
        }
    }
}
