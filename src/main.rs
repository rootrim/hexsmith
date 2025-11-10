mod app;
mod ui;

use anyhow::Context;
use app::App;
use ratatui;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut terminal = ratatui::init();

    let binary_path = std::env::args()
        .nth(1)
        .context("Please provide a binary to analyze")?;

    let mut app = App::new(binary_path);

    app.run(&mut terminal)?;

    ratatui::restore();

    Ok(())
}
