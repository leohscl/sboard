use tui_textarea::TextArea;
use tui_textarea::{CursorMove, Scrolling};

pub struct Editor<'a> {
    pub textarea: TextArea<'a>,
}

impl<'a> Editor<'a> {
    pub fn new(text: &str) -> Self {
        let vec_lines: Vec<String> = text.split('\n').map(|s| s.to_string()).collect();
        Editor {
            textarea: TextArea::new(vec_lines),
        }
    }

    pub fn send_char(&mut self, c_sent: char) {
        match c_sent {
            'h' => self.textarea.move_cursor(CursorMove::Back),
            'j' => self.textarea.move_cursor(CursorMove::Down),
            'k' => self.textarea.move_cursor(CursorMove::Up),
            'l' => self.textarea.move_cursor(CursorMove::Forward),
            'w' => self.textarea.move_cursor(CursorMove::WordForward),
            'b' => self.textarea.move_cursor(CursorMove::WordBack),
            '^' => self.textarea.move_cursor(CursorMove::Head),
            '$' => self.textarea.move_cursor(CursorMove::End),
            'e' => self.textarea.scroll((1, 0)),
            'y' => self.textarea.scroll((-1, 0)),
            'D' => self.textarea.scroll(Scrolling::HalfPageDown),
            'U' => self.textarea.scroll(Scrolling::HalfPageUp),
            'F' => self.textarea.scroll(Scrolling::PageDown),
            'B' => self.textarea.scroll(Scrolling::PageUp),
            'g' => self.textarea.move_cursor(CursorMove::Top),
            'G' => self.textarea.move_cursor(CursorMove::Bottom),
            _ => (),
        }
    }
}
