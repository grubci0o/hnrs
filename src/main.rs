mod api;
mod tui;

use std::ffi::c_long;
use std::io;
use std::io::Read;
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use api::HNApi;
use tui::App;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api = HNApi::new();
    let ids = api.fetch_top_ids().await?;
    println!("Got {} top stories", ids.len());

    let first_story = api.fetch_item(ids[0]).await?;
    println!("First story: {:?}", first_story.text);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen);
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let top_stories = api.fetch_top_stories().await?;
    let mut app = App::new(api, top_stories);
    let res = app.run(&mut terminal);
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    res
}