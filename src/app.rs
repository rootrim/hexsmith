use std::process::Stdio;

use crate::event::Event;
use anyhow::anyhow;
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use tokio::io::AsyncReadExt;
use tokio::process::{Child, Command};

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

pub struct Process {
    pub path: String,
    pub child: Child,
    pub output_buffer: [u8; 1024],
    pub input_buffer: Vec<u8>,
}

impl Process {
    pub fn new(path: String) -> Self {
        let child = Command::new(&path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to spawn process");
        let output_buffer = [0u8; 1024];
        let input_buffer = Vec::new();

        Self {
            path,
            child,
            output_buffer,
            input_buffer,
        }
    }

    pub async fn read_output(&mut self) -> anyhow::Result<usize> {
        if let Some(stdout) = &mut self.child.stdout {
            let n = stdout.read(&mut self.output_buffer).await?;
            Ok(n)
        } else {
            Err(anyhow!("No stdout available"))
        }
    }
}
