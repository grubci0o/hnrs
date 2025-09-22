mod api;

use std::io::Read;
use api::HNApi;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api = HNApi::new();
    let ids = api.fetch_top_ids().await?;
    println!("Got {} top stories", ids.len());

    let first_story = api.fetch_item(ids[0]).await?;
    println!("First story: {:?}", first_story.text);

    Ok(())
}