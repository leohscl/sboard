use crate::app::App;
use crate::app::DisplayState;
use crate::app::{DESCRIPTION_JOB, DESCRIPTION_LOG};
use crate::editor::Editor;
use crate::jobs::job_parser::JobFields;
use ratatui::prelude::*;
use ratatui::widgets::block::Position;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::List;
use ratatui::widgets::ListItem;
use ratatui::widgets::{Row, Table};
use ratatui::Frame;
use tui_popup::Popup;

use tracing::info;

struct ColoredString {
    string: String,
    color: Color,
}

fn display_jobs(frame: &mut Frame, app: &App) {
    if let DisplayState::Jobs(ref job_info) = app.display_state {
        let colored_strings: Vec<ColoredString> = job_info
            .job_display
            .iter()
            .map(|job_fields| ColoredString {
                string: job_fields.display_lines(job_info.efficiency_display),
                color: job_fields.state.to_color(),
            })
            .collect();
        let list_items = build_list(&colored_strings, app.highlighted);
        let legend = DESCRIPTION_JOB.to_string();
        let list_widget = build_widget(list_items, &legend);
        frame.render_widget(list_widget, frame.size());
    }
}

fn build_list(lines: &[ColoredString], highlighted: Option<usize>) -> Vec<ListItem> {
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

fn display_details(frame: &mut Frame, app: &App, log_files: &[String]) {
    let colored_strings: Vec<_> = log_files
        .iter()
        .map(|s| ColoredString {
            string: s.clone(),
            color: Color::White,
        })
        .collect();
    let list_items = build_list(&colored_strings, app.highlighted);
    let list_widget = build_widget(list_items, DESCRIPTION_LOG);
    frame.render_widget(list_widget, frame.size());
}

// fn display_report(frame: &mut Frame, job_fields: &JobFields) {
//     let maxrss = job_fields.maxrss.clone().take().unwrap();
//     let reqmem = job_fields.reqmem.clone().take().unwrap();
//     // info!("maxrss: {}, reqmem: {}", maxrss, reqmem);
//     let memory_eff = (maxrss as f64 / reqmem as f64) * 100f64;
//     let formated_mem_eff = format!("{:.1$}", memory_eff, 1);
//     let rows = [Row::new(vec![formated_mem_eff + "%"])];
//     // Columns widths are constrained in the same way as Layout...
//     let widths = [Constraint::Length(10)];
//     let table = Table::new(rows, widths)
//         .column_spacing(10)
//         .style(Style::new().red())
//         .header(
//             Row::new(vec!["MemEff"])
//                 .style(Style::new().bold())
//                 .bottom_margin(1),
//         );
//     frame.render_widget(table, frame.size());
// }

pub fn ui(frame: &mut Frame, app: &App) {
    match &app.display_state {
        DisplayState::Editor(ref editor) => display_editor(frame, editor),
        DisplayState::Jobs(_) => display_jobs(frame, app),
        DisplayState::Logs(ref details) => display_details(frame, app, details),
        DisplayState::Empty => (),
        // DisplayState::Report(job_detail) => display_report(frame, job_detail),
    }
    display_popup(frame, app);
}

fn display_popup(frame: &mut Frame, app: &App) {
    if let Some(ref my_popup) = app.popup {
        let area = frame.size();
        let popup = Popup::new(my_popup.popup_text.clone(), "Press any key to exit");
        frame.render_widget(popup.to_widget(), area);
    }
}

pub trait Colorable {
    fn to_color(&self) -> Color;
}

enum LineType {
    Normal,
    Highlighted,
}

impl Colorable for LineType {
    fn to_color(&self) -> Color {
        match self {
            LineType::Normal => Color::Black,
            LineType::Highlighted => Color::DarkGray,
        }
    }
}

fn build_list_item(c_str: &ColoredString, line_type: LineType) -> ListItem {
    let bg_color = line_type.to_color();
    ListItem::new(c_str.string.clone()).style(Style::default().fg(c_str.color).bg(bg_color))
}
