use crossterm::style::Color;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::Stylize,
    widgets::{Block, BorderType, Paragraph, Widget},
};

use crate::app::App;

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(area);
        let chunks = split;

        let text = "Habbab".to_string();

        let terminal_block = Block::bordered()
            .title("Terminal")
            .border_type(BorderType::Rounded);
        let other_block = Block::bordered()
            .title("Other Block")
            .border_type(BorderType::Rounded);

        let paragraph = Paragraph::new(text)
            .block(terminal_block)
            .fg(Color::Cyan)
            .bg(Color::Black);

        paragraph.render(chunks[0], buf);
        other_block.render(chunks[1], buf);
    }
}
