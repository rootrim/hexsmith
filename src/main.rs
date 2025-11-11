use anyhow::Context;

use crate::app::App;

pub mod app;
pub mod event;
pub mod ui;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let arg1 = std::env::args()
        .nth(1)
        .context("Please provide a path to the target process")?;

    let terminal = ratatui::init();
    let result = App::new(arg1).run(terminal).await;
    ratatui::restore();
    result
}
