use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders, Padding, BorderType},
};

pub fn bordered() -> Block<'static> {
    Block::default().border_type(BorderType::Rounded).borders(Borders::ALL)
}

pub fn titled(title: impl Into<String>) -> Block<'static> {
    bordered().title(title.into())
}

pub fn content_block(title: impl Into<String>) -> Block<'static> {
    titled(title).padding(Padding::uniform(1))
}

pub fn styled_block(title: impl Into<String>, style: Style) -> Block<'static> {
    titled(title).style(style)
}

pub fn error_block(title: impl Into<String>) -> Block<'static> {
    styled_block(title, Style::default().fg(Color::Red))
}
