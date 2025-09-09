use crate::block_styles::{titled, error_block};
use crate::ui_utils::centered_rect_fixed_height;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Clear, Paragraph, Wrap},
    Frame,
};

#[derive(Debug, Clone)]
pub enum ModalStyle {
    Normal,
    Warning,
    Error,
}

#[derive(Debug)]
pub struct Modal<'a> {
    title: &'a str,
    style: ModalStyle,
    width_percentage: u16,
    height: u16,
    content: Vec<ModalContent<'a>>,
}

#[derive(Debug)]
pub enum ModalContent<'a> {
    Text {
        text: &'a str,
        alignment: Alignment,
        style: Style,
    },
    Spacing(u16),
}

impl<'a> Modal<'a> {
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            style: ModalStyle::Normal,
            width_percentage: 60,
            height: 7,
            content: Vec::new(),
        }
    }

    pub fn style(mut self, style: ModalStyle) -> Self {
        self.style = style;
        self
    }

    pub fn size(mut self, width_percentage: u16, height: u16) -> Self {
        self.width_percentage = width_percentage;
        self.height = height;
        self
    }

    pub fn add_text(mut self, text: &'a str, alignment: Alignment, style: Style) -> Self {
        self.content.push(ModalContent::Text {
            text,
            alignment,
            style,
        });
        self
    }


    pub fn add_spacing(mut self, height: u16) -> Self {
        self.content.push(ModalContent::Spacing(height));
        self
    }

    pub fn render(&self, f: &mut Frame, background_area: Rect) -> Rect {
        let popup_area = centered_rect_fixed_height(self.width_percentage, self.height, background_area);
        f.render_widget(Clear, popup_area);

        let background_block = match self.style {
            ModalStyle::Normal => titled(self.title),
            ModalStyle::Warning => titled(self.title).style(Style::default().fg(Color::Yellow)),
            ModalStyle::Error => error_block(self.title),
        };
        f.render_widget(background_block, popup_area);

        // Create constraints for content
        let constraints: Vec<Constraint> = self.content.iter().map(|content| {
            match content {
                ModalContent::Text { .. } => Constraint::Length(2),
                ModalContent::Spacing(height) => Constraint::Length(*height),
            }
        }).collect();

        if constraints.is_empty() {
            return popup_area;
        }

        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(popup_area);

        // Render content
        for (i, content) in self.content.iter().enumerate() {
            if i < content_chunks.len() {
                match content {
                    ModalContent::Text { text, alignment, style } => {
                        let paragraph = Paragraph::new(*text)
                            .alignment(*alignment)
                            .style(*style)
                            .wrap(Wrap { trim: true });
                        f.render_widget(paragraph, content_chunks[i]);
                    }
                    ModalContent::Spacing(_) => {
                        // Spacing is handled by the layout constraints
                    }
                }
            }
        }

        popup_area
    }
}