use crate::event::Event;
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::event::{AppEvent, EventHandler};

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub events: EventHandler,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: false,
            events: EventHandler::new(),
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> anyhow::Result<()> {
        while self.running {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            match self.events.next().await? {
                Event::Tick => self.tick(),
                Event::Crossterm(event) => match event {
                    crossterm::event::Event::Key(key_event)
                        if key_event.kind == KeyEventKind::Press =>
                    {
                        self.handle_key_event(key_event)?
                    }
                    _ => {}
                },
                Event::App(app_event) => match app_event {
                    AppEvent::Quit => self.running = false,
                },
            }
        }
        Ok(())
    }

    pub fn tick(&self) {}

    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<()> {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Quit),

            _ => {}
        }

        Ok(())
    }
}
