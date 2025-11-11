use std::process::Stdio;

use crate::event::Event;
use anyhow::anyhow;
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::{Child, Command};

use crate::event::{AppEvent, EventHandler};

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub events: EventHandler,
    pub current_pane: Pane,
    pub target: Process,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            events: EventHandler::new(),
            current_pane: Pane::Terminal,
            target: Process::new(
                "/nix/store/qx4mns4a0nzcv783jbvmgs0wgv9fxpks-system-path/bin/ls".into(),
            ),
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

#[derive(Debug)]
pub struct Process {
    pub path: String,
    pub child: Child,
    pub output_buffer: Vec<u8>,
}

impl Process {
    pub fn new(path: String) -> Self {
        let child = Command::new(&path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to spawn process");
        let output_buffer = Vec::new();

        Self {
            path,
            child,
            output_buffer,
        }
    }

    pub async fn read_output(&mut self) -> anyhow::Result<usize> {
        if let Some(stdout) = &mut self.child.stdout {
            let mut buf = [0u8; 1024];
            let n = stdout.read(&mut buf).await?;
            self.output_buffer.extend_from_slice(&buf[..n]);
            Ok(n)
        } else {
            Err(anyhow!("No stdout available"))
        }
    }

    pub async fn write_input(&mut self, data: &[u8]) -> anyhow::Result<()> {
        if let Some(stdin) = &mut self.child.stdin {
            stdin.write_all(data).await?;
            stdin.flush().await?;
            Ok(())
        } else {
            Err(anyhow!("No stdin available"))
        }
    }

    pub fn get_output_as_string(&mut self) -> String {
        String::from_utf8_lossy(&self.output_buffer).to_string()
    }
}
