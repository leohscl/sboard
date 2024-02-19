use crate::app::App;
use crate::app::DisplayState;
use crate::app::JobDetails;
use crate::editor::Editor;
use ratatui::prelude::*;
use ratatui::widgets::block::Position;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::List;
use ratatui::widgets::ListItem;
use ratatui::Frame;

fn display_jobs(frame: &mut Frame, app: &App) {
    if let DisplayState::Jobs(ref job_list) = app.display_state {
        let list_items = build_list(&job_list.jobs, app.highlighted);
        let list_widget = build_widget(list_items, "[q]uit [t]oggle_refresh [l]ogs");
        frame.render_widget(list_widget, frame.size());
    }
}

fn display_details(frame: &mut Frame, app: &App, details: &JobDetails) {
    let strings = [details.err_file.clone(), details.log_file.clone()].to_vec();
    let list_items = build_list(&strings, app.highlighted);
    let list_widget = build_widget(list_items, "[q]uit [v]iew");
    frame.render_widget(list_widget, frame.size());
}

fn build_list(lines: &[String], highlighted: Option<usize>) -> Vec<ListItem> {
    lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            if let Some(highlighted_i) = highlighted {
                if highlighted_i == i {
                    build_list_item(line, LineType::Highlighted)
                } else {
                    build_list_item(line, LineType::Normal)
                }
            } else {
                build_list_item(line, LineType::Normal)
            }
        })
        .collect()
}

fn build_widget<'a>(list_items: Vec<ListItem<'a>>, text: &'a str) -> List<'a> {
    List::new(list_items)
        .block(
            Block::default()
                .title(text)
                .title_position(Position::Bottom)
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
        .highlight_symbol(">>")
}

fn display_editor(frame: &mut Frame, editor: &Editor) {
    let widget = editor.textarea.widget();
    frame.render_widget(widget, frame.size());
}

pub fn ui(frame: &mut Frame, app: &App) {
    match &app.display_state {
        DisplayState::Editor(ref editor) => display_editor(frame, editor),
        DisplayState::Jobs(_) => display_jobs(frame, app),
        DisplayState::Details(ref details) => display_details(frame, app, details),
        DisplayState::Empty => (),
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
