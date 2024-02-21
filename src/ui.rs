use crate::app::App;
use crate::app::DisplayState;
use crate::app::JobTime;
use crate::editor::Editor;
use ratatui::prelude::*;
use ratatui::widgets::block::Position;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::List;
use ratatui::widgets::ListItem;
use ratatui::Frame;

fn display_jobs(frame: &mut Frame, app: &App) {
    if let DisplayState::Jobs(ref job_info) = app.display_state {
        let display_str = job_info
            .job_list
            .iter()
            .map(|job_fields| job_fields.display_lines())
            .collect();
        let list_items = build_list(&display_str, app.highlighted);
        let option = if matches!(job_info.time, JobTime::Past) {
            "[c]urrent"
        } else {
            "[p]ast"
        };
        let legend = "[q]uit [t]oggle_refresh ".to_string() + option;
        let list_widget = build_widget(list_items, &legend);
        frame.render_widget(list_widget, frame.size());
    }
}

fn build_list(lines: &Vec<String>, highlighted: Option<usize>) -> Vec<ListItem> {
    lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            if let Some(highlighted_i) = highlighted {
                if highlighted_i == i {
                    build_list_item(&line, LineType::Highlighted)
                } else {
                    build_list_item(&line, LineType::Normal)
                }
            } else {
                build_list_item(&line, LineType::Normal)
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

fn display_details(frame: &mut Frame, app: &App, log_files: &Vec<String>) {
    let list_items = build_list(&log_files, app.highlighted);
    let list_widget = build_widget(list_items, "[q]uit [v]iew");
    frame.render_widget(list_widget, frame.size());
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
    let font_color = match line_type {
        LineType::Normal => Color::White,
        LineType::Highlighted => Color::Yellow,
    };
    ListItem::new(line).style(Style::default().fg(font_color).bg(Color::Black))
}
