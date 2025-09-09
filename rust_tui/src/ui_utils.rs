use ratatui::layout::{Constraint, Direction, Layout};

pub fn centered_rect_fixed_height(
    percent_x: u16,
    height_lines: u16,
    r: ratatui::layout::Rect,
) -> ratatui::layout::Rect {
    let remaining_height = r.height.saturating_sub(height_lines);
    let top_padding = remaining_height / 2;
    let bottom_padding = remaining_height - top_padding;
    
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(top_padding),
            Constraint::Length(height_lines),
            Constraint::Length(bottom_padding),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
