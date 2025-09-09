use crate::block_styles::{bordered, titled};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::Paragraph,
    Frame,
};

pub struct StatusBar<'a> {
    character_count: usize,
    character_limit: usize,
    button_text: &'a str,
    show_button: bool,
    thresholds: CharacterThresholds,
}

#[derive(Debug, Clone)]
pub struct CharacterThresholds {
    pub warning: usize,
    pub caution: usize,
    pub danger: usize,
}

impl Default for CharacterThresholds {
    fn default() -> Self {
        Self {
            warning: 300,
            caution: 350,
            danger: 400,
        }
    }
}

impl<'a> StatusBar<'a> {
    pub fn new(character_count: usize, character_limit: usize, button_text: &'a str) -> Self {
        Self {
            character_count,
            character_limit,
            button_text,
            show_button: true,
            thresholds: CharacterThresholds::default(),
        }
    }

    pub fn with_thresholds(mut self, thresholds: CharacterThresholds) -> Self {
        self.thresholds = thresholds;
        self
    }

    pub fn show_button(mut self, show: bool) -> Self {
        self.show_button = show;
        self
    }

    pub fn render(&self, f: &mut Frame, area: Rect) -> Option<Rect> {
        let constraints = if self.show_button {
            vec![Constraint::Percentage(70), Constraint::Percentage(30)]
        } else {
            vec![Constraint::Percentage(100)]
        };

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(area);

        // Character count with color coding
        let char_count_text = format!("Characters: {}/{}", self.character_count, self.character_limit);
        let char_count_style = if self.character_count > self.character_limit {
            Style::default().fg(Color::Red)
        } else if self.character_count > self.thresholds.danger {
            Style::default().fg(Color::Rgb(255, 191, 0)) // amber
        } else if self.character_count > self.thresholds.caution {
            Style::default().fg(Color::Rgb(255, 255, 0)) // yellow
        } else if self.character_count > self.thresholds.warning {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let char_count = Paragraph::new(char_count_text)
            .block(titled("Status"))
            .style(char_count_style);
        f.render_widget(char_count, chunks[0]);

        // Submit button (if enabled)
        if self.show_button && chunks.len() > 1 {
            let submit_button = Paragraph::new(format!("[ {} ]", self.button_text))
                .block(bordered())
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Green));
            f.render_widget(submit_button, chunks[1]);
            Some(chunks[1])
        } else {
            None
        }
    }
}