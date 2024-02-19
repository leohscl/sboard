use tui_textarea::Input;
use tui_textarea::Key;
use tui_textarea::TextArea;

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
        let input = Input {
            key: Key::Char(c_sent),
            ctrl: true,
            alt: false,
            shift: false,
        };
        self.textarea.input(input);
    }
}
