use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::process::Stdio;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};

use crate::ui::ui;

pub struct App {
    pub process: Process,
    pub current_pane: CurrentPane,
    pub running: bool,
}

pub enum CurrentPane {
    Terminal,
    Other,
}

#[allow(dead_code)]
pub struct Process {
    pub path: String,
    pub child: Child,
    pub lines: Arc<Mutex<Vec<String>>>,
}

impl App {
    const FRAMES_PER_SECOND: f32 = 120.0;

    pub fn new(binary_path: String) -> Self {
        let mut child = Command::new(&binary_path)
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()
            .unwrap();

        let stdout = child.stdout.take().unwrap();
        let mut reader = BufReader::new(stdout).lines();
        let lines = Arc::new(Mutex::new(Vec::new()));

        let lines_ref = lines.clone();
        tokio::spawn(async move {
            while let Ok(Some(line)) = reader.next_line().await {
                lines_ref.lock().unwrap().push(line);
            }
        });

        App {
            process: Process {
                path: binary_path,
                child,
                lines,
            },
            current_pane: CurrentPane::Other,
            running: true,
        }
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
        let period = Duration::from_secs_f32(1.0 / Self::FRAMES_PER_SECOND);
        let mut interval = tokio::time::interval(period);

        while self.running {
            tokio::select! {
                _ = interval.tick() => { terminal.draw(|f| ui(f, self))?;},
            }

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Release {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') => self.running = false,
                    KeyCode::Left | KeyCode::Char('h') => self.current_pane = CurrentPane::Terminal,
                    KeyCode::Right | KeyCode::Char('l') => self.current_pane = CurrentPane::Other,
                    _ => {}
                }
            }
        }
        Ok(())
    }

    pub fn handle_event(&mut self, _event: Event) {
        // Handle other events if necessary
    }
}
