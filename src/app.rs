use std::process::Stdio;

use crate::event::Event;
use anyhow::anyhow;
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use tokio::io::AsyncWriteExt;
use tokio::io::{AsyncReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;

use crate::event::{AppEvent, EventHandler};

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub events: EventHandler,
    pub current_pane: Pane,
    pub target: Process,
    pub output_rx: mpsc::UnboundedReceiver<String>,
}

impl App {
    pub fn new(path: String) -> Self {
        let mut target = Process::new(path);
        let output_rx = target.spawn_reader();

        Self {
            running: true,
            events: EventHandler::new(),
            current_pane: Pane::Terminal,
            target,
            output_rx,
        }
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> anyhow::Result<()> {
        while self.running {
            while let Ok(chunk) = self.output_rx.try_recv() {
                self.target
                    .output_buffer
                    .extend_from_slice(chunk.as_bytes());
            }
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

    pub async fn write_input(&mut self, data: &[u8]) -> anyhow::Result<()> {
        if let Some(stdin) = &mut self.child.stdin {
            stdin.write_all(data).await?;
            stdin.flush().await?;
            Ok(())
        } else {
            Err(anyhow!("No stdin available"))
        }
    }

    pub fn get_output_as_string(&self) -> String {
        String::from_utf8_lossy(&self.output_buffer).to_string()
    }

    pub fn spawn_reader(&mut self) -> mpsc::UnboundedReceiver<String> {
        let stdout = self.child.stdout.take().expect("No stdout");
        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);
            let mut buf = [0u8; 1024];

            loop {
                match reader.read(&mut buf).await {
                    Ok(0) => break,
                    Ok(n) => {
                        let chunk = String::from_utf8_lossy(&buf[..n]).to_string();
                        if tx.send(chunk).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        rx
    }
}
