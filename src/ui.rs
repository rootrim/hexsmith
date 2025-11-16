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
        let otherchunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
            .split(chunks[1]);

        let terminal_style = if let Pane::Terminal = self.current_pane {
            Style::default().bg(Color::DarkGray)
        } else {
            Style::default()
        };

        let payload_style = if let Pane::Payload = self.current_pane {
            Style::default().bg(Color::DarkGray)
        } else {
            Style::default()
        };

        let shellcode_style = if let Pane::ShellCode = self.current_pane {
            Style::default().bg(Color::DarkGray)
        } else {
            Style::default()
        };

        let terminal = Block::bordered()
            .title("Terminal")
            .border_type(BorderType::Rounded)
            .style(terminal_style);
        let payload = Block::bordered()
            .title("Payload Block")
            .border_type(BorderType::Rounded)
            .style(payload_style);
        let shellcode = Block::bordered()
            .title("ShellCode Block")
            .border_type(BorderType::Rounded)
            .style(shellcode_style);

        let terminal = Paragraph::new(self.pty_buffer.clone()).block(terminal);
        let payload = Paragraph::new(self.payload_buffer.clone()).block(payload);
        let shellcode = Paragraph::new(self.shellcode_buffer.clone()).block(shellcode);

        terminal.render(chunks[0], buf);
        payload.render(otherchunks[1], buf);
        shellcode.render(otherchunks[0], buf);
    }
}
