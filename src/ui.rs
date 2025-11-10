use crate::app::{App, CurrentPane};
use anyhow::Context;
use ratatui::text;

use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn ui(frame: &mut ratatui::Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(frame.area());

    let terminal_style = if let CurrentPane::Terminal = app.current_pane {
        Style::default().bg(Color::DarkGray)
    } else {
        Style::default()
    };

    let other_style = if let CurrentPane::Other = app.current_pane {
        Style::default().bg(Color::DarkGray)
    } else {
        Style::default()
    };

    let terminal_block = Block::default()
        .title("Terminal")
        .borders(Borders::ALL)
        .style(terminal_style);
    let other_block = Block::default()
        .title("Other Pane")
        .borders(Borders::ALL)
        .style(other_style);

    let terminal = Paragraph::new("Habbab").block(terminal_block);

    frame.render_widget(terminal, chunks[0]);
    frame.render_widget(other_block, chunks[1]);
}
