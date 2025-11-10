use crate::event::Event;
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::event::{AppEvent, EventHandler};

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub events: EventHandler,
    pub current_pane: Pane,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            events: EventHandler::new(),
            current_pane: Pane::Terminal,
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
                    AppEvent::PaneSwitch => {
                        self.current_pane = match self.current_pane {
                            Pane::Terminal => Pane::Other,
                            Pane::Other => Pane::Terminal,
                        }
                    }
                },
            }
        }
        Ok(())
    }

    pub fn tick(&self) {}

    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<()> {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Quit),
            KeyCode::Tab => self.events.send(AppEvent::PaneSwitch),
            _ => {}
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum Pane {
    Terminal,
    Other,
}
