use anyhow::Context;
use futures::{FutureExt, StreamExt};
use ratatui::crossterm::event::Event as CrosstermEvent;
use std::time::Duration;
use tokio::sync::mpsc;

const TICK_FPS: f64 = 30.0;

pub enum Event {
    Tick,
    Crossterm(CrosstermEvent),
    App(AppEvent),
}

pub enum AppEvent {
    Quit,
    PaneSwitch,
}

pub struct EventHandler {
    sender: mpsc::UnboundedSender<Event>,
    receiver: mpsc::UnboundedReceiver<Event>,
}

impl EventHandler {
    pub fn new() -> Self {
        EventHandler::default()
    }

    pub async fn next(&mut self) -> anyhow::Result<Event> {
        self.receiver
            .recv()
            .await
            .context("Failed to receive event")
    }

    pub fn send(&mut self, app_event: AppEvent) {
        let _ = self.sender.send(Event::App(app_event));
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let actor = EventTask::new(sender.clone());
        tokio::spawn(async { actor.run().await });
        Self { sender, receiver }
    }
}

/// A thread that handles reading crossterm events and emitting tick events on a regular schedule.
struct EventTask {
    /// Event sender channel.
    sender: mpsc::UnboundedSender<Event>,
}

impl EventTask {
    /// Constructs a new instance of [`EventThread`].
    fn new(sender: mpsc::UnboundedSender<Event>) -> Self {
        Self { sender }
    }

    /// Runs the event thread.
    ///
    /// This function emits tick events at a fixed rate and polls for crossterm events in between.
    async fn run(self) -> anyhow::Result<()> {
        let tick_rate = Duration::from_secs_f64(1.0 / TICK_FPS);
        let mut reader = crossterm::event::EventStream::new();
        let mut tick = tokio::time::interval(tick_rate);
        loop {
            let tick_delay = tick.tick();
            let crossterm_event = reader.next().fuse();
            tokio::select! {
              _ = self.sender.closed() => {
                break;
              }
              _ = tick_delay => {
                self.send(Event::Tick);
              }
              Some(Ok(evt)) = crossterm_event => {
                self.send(Event::Crossterm(evt));
              }
            };
        }
        Ok(())
    }

    fn send(&self, event: Event) {
        // Ignores the result because shutting down the app drops the receiver, which causes the send
        // operation to fail. This is expected behavior and should not panic.
        let _ = self.sender.send(event);
    }
}
