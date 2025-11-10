use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::process::Stdio;
use tokio::io::{BufReader, BufWriter};
use tokio::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command};

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

pub struct Process {
    path: String,
    child: Child,
    reader: BufReader<ChildStdout>,
    writer: BufWriter<ChildStdin>,
}

impl App {
    pub fn new(binary_path: String) -> Self {
        let mut child = Command::new(&binary_path)
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()
            .unwrap();

        let stdout = //crazy
            child.stdout.take().unwrap();

        let stdin = //crazy
            child.stdin.take().unwrap();

        let reader = //nani
            BufReader::new(stdout);

        let writer = //nani
            BufWriter::new(stdin);

        let app = App {
            process: Process {
                path: binary_path,
                child,
                reader,
                writer,
            },
            current_pane: CurrentPane::Terminal,
            running: true,
        };
        app
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
        while self.running {
            terminal.draw(|f| ui(f, &self))?;
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
}
