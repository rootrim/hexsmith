use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Paragraph, Widget},
};

use crate::app::{App, Pane};

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(area);

        let terminal_style = if let Pane::Terminal = self.current_pane {
            Style::default().bg(Color::DarkGray)
        } else {
            Style::default()
        };

        let other_style = if let Pane::Other = self.current_pane {
            Style::default().bg(Color::DarkGray)
        } else {
            Style::default()
        };

        let terminal_block = Block::bordered()
            .title("Terminal")
            .border_type(BorderType::Rounded)
            .style(terminal_style);
        let other_block = Block::bordered()
            .title("Other Block")
            .border_type(BorderType::Rounded)
            .style(other_style);

        let paragraph = Paragraph::new(self.pty_buffer.clone()).block(terminal_block);

        paragraph.render(chunks[0], buf);
        other_block.render(chunks[1], buf);
    }
}
