use std::io::Read;

use crate::event::Event;
use anyhow::Context;
use portable_pty::{CommandBuilder, PtyPair, PtySize, native_pty_system};
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use tokio::sync::mpsc::{Receiver, Sender, channel};

use crate::event::{AppEvent, EventHandler};

type PtyChannels = (PtyPair, Sender<Vec<u8>>, Receiver<Vec<u8>>);

pub struct App {
    pub running: bool,
    pub events: EventHandler,
    pub current_pane: Pane,
    pub pair: PtyPair,
    pub pty_buffer: String,
    pub tx: Sender<Vec<u8>>,
    pub rx: Receiver<Vec<u8>>,
    pub shellcode_buffer: String,
    pub payload_buffer: String,
}

impl App {
    pub fn new(path: String) -> Self {
        let (pair, tx, rx) = App::create_pty_process(path)
            .context("Failed to create pty process")
            .unwrap();
        Self {
            running: true,
            events: EventHandler::new(),
            current_pane: Pane::Terminal,
            pair,
            pty_buffer: String::new(),
            tx,
            rx,
            shellcode_buffer: String::new(),
            payload_buffer: String::new(),
        }
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> anyhow::Result<()> {
        while self.running {
            while let Ok(data) = self.rx.try_recv() {
                self.pty_buffer.push_str(&String::from_utf8_lossy(&data));
                let lines: Vec<&str> = self.pty_buffer.lines().collect();
                if lines.len() > 100 {
                    self.pty_buffer = lines[lines.len() - 100..].join("\n");
                }
            }
            terminal.draw(|frame| {
                let area = frame.area();
                self.resize_pty(area.width / 2, area.height).unwrap_or(());
                frame.render_widget(&self, area)
            })?;
            match self.events.next().await? {
                Event::Tick => self.tick(),
                Event::Crossterm(event) => match event {
                    crossterm::event::Event::Key(key_event)
                        if key_event.kind == KeyEventKind::Press =>
                    {
                        self.handle_key_event(key_event).await?
                    }
                    _ => {}
                },
                Event::App(app_event) => match app_event {
                    AppEvent::Quit => self.running = false,
                    AppEvent::PaneSwitch => {
                        self.current_pane = match self.current_pane {
                            Pane::Terminal => Pane::ShellCode,
                            Pane::ShellCode => Pane::Payload,
                            Pane::Payload => Pane::Terminal,
                        }
                    }
                },
            }
        }
        Ok(())
    }

    pub fn tick(&self) {}

    pub async fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<()> {
        match key_event.code {
            KeyCode::Esc => self.events.send(AppEvent::Quit),
            KeyCode::Tab => self.events.send(AppEvent::PaneSwitch),
            KeyCode::Char(c) => match self.current_pane {
                Pane::Terminal => self.tx.send(vec![c as u8]).await?,
                Pane::ShellCode => self.shellcode_buffer.push(c),
                Pane::Payload => self.payload_buffer.push(c),
            },
            KeyCode::Enter => match self.current_pane {
                Pane::Terminal => self.tx.send(vec![b'\n']).await?,
                _ => self.send_code().await?,
            },
            _ => {}
        }

        Ok(())
    }

    pub fn create_pty_process(path: String) -> anyhow::Result<PtyChannels> {
        let (tx_input, mut rx_input) = channel::<Vec<u8>>(100);
        let (tx_output, rx_output) = channel::<Vec<u8>>(100);

        let pair = native_pty_system()
            .openpty(PtySize {
                rows: 0,
                cols: 0,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("Failed to open pty")
            .unwrap();

        pair.slave
            .spawn_command(CommandBuilder::new(path))
            .context("Failed to spawn command in pty")
            .unwrap();

        let mut reader = pair
            .master
            .try_clone_reader()
            .context("Failed to clone pty reader")?;

        tokio::spawn(async move {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        if tx_output.send(buf[..n].to_vec()).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        let mut writer = pair.master.take_writer()?;
        tokio::spawn(async move {
            while let Some(data) = rx_input.recv().await {
                let _ = writer.write_all(&data);
            }
        });

        Ok((pair, tx_input, rx_output))
    }

    pub fn resize_pty(&mut self, cols: u16, rows: u16) -> anyhow::Result<()> {
        self.pair
            .master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("Failed to resize pty")?;
        Ok(())
    }

    pub async fn send_code(&mut self) -> anyhow::Result<()> {
        let shellcode_as_bytes: Vec<u8> = self
            .shellcode_buffer
            .clone()
            .split("\\x")
            .filter(|s| !s.is_empty())
            .map(|h| {
                u8::from_str_radix(&h[..2], 16)
                    .context("Couldn't convert the shellcode buffer to a Vec<u8>")
                    .unwrap()
            })
            .collect();
        let payload_as_bytes: Vec<u8> = self
            .payload_buffer
            .clone()
            .split("\\x")
            .filter(|s| !s.is_empty())
            .map(|h| {
                u8::from_str_radix(&h[..2], 16)
                    .context("Couldn't convert the payload buffer to a Vec<u8>")
                    .unwrap()
            })
            .collect();
        let mut the_whole_payload: Vec<u8> = Vec::new();
        the_whole_payload.extend(payload_as_bytes);
        the_whole_payload.extend(shellcode_as_bytes);
        self.tx
            .send(the_whole_payload)
            .await
            .context("Couldn't send the payload")?;
        Ok(())
    }
}

pub enum Pane {
    Terminal,
    ShellCode,
    Payload,
}
