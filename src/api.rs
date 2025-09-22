use reqwest::Client;
use anyhow::Result;
use serde::Deserialize;
use html2text::from_read;

static HN_TOP_IDS_URL:&'static str = "https://hacker-news.firebaseio.com/v0";
pub struct HNApi {
    client: Client,
    base_url: &'static str,
}

#[derive(Debug, Clone)]
pub struct CommentNode {
    pub id: u64,
    pub depth: usize,
    pub expanded: bool,
    pub loading: bool,
    pub item: Option<HNItem>,
    pub children: Vec<CommentNode>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HNItem {
    pub id: u64,
    pub by: Option<String>,
    pub title: Option<String>,
    pub url: Option<String>,
    pub kids: Option<Vec<u64>>,
    pub score: Option<u64>,
    pub time: Option<u64>,
    pub text: Option<String>,
    pub r#type: Option<String>,
}

impl HNApi {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("hn-tui (Rust)")
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            base_url: HN_TOP_IDS_URL
        }
    }

    pub async fn fetch_top_ids(&self) -> Result<Vec<u64>> {
        let url = format!("{}/topstories.json", self.base_url);
        let ids = self.client.get(url).send().await?.json::<Vec<u64>>().await?;
        Ok(ids)
    }

    pub async fn fetch_item(&self, id: u64) -> Result<HNItem> {
        let url = format!("{}/item/{id}.json", self.base_url);
        let item = self.client.get(url).send().await?.json::<HNItem>().await?;
        Ok(item)
    }

    pub async fn fetch_comment(&self, id: u64, terminal_width: usize) -> Result<HNItem> {
        let mut item = self.fetch_item(id).await?;
        if let Some(raw) = item.text.take() {
            let plain_text = from_read(raw.as_bytes(), terminal_width)?;
            let cleaned = plain_text
                .lines()
                .map(|line| line.trim_end())
                .collect::<Vec<_>>()
                .join("\n")
                .trim()
                .to_string();

            item.text = Some(cleaned);
        }

        Ok(item)
    }

    pub async fn expand_node(&self, node: &mut CommentNode, width: usize) -> Result<()> {
        if node.expanded {
            node.expanded = false;
            node.children.clear();
            return Ok(())
        }

        node.loading = true;

        if node.item.is_none() {
            match self.fetch_comment(node.id, width) {
                Ok(item) => {
                    node.item = item;
                }
                Err(e) => {
                    node.loading = false;
                    return Err(e)
                }
            }
        }

        if let Some(item) = &node.item {
            if let Some(kids) = &item.kids {
                node.children = kids
                    .iter()
                    .map(|&kid_id| CommentNode::placeholder(kid_id, node.depth + 1))
                    .collect();
            } else {
                node.children.clear();
            }
        }

        node.expanded = true;
        node.loading = false;
        Ok(())
    }
}

impl CommentNode {
    pub fn placeholder(id: u64, depth: usize) -> Self {
        Self{
            id,
            depth,
            expanded: false,
            loading: false,
            item: None,
            children: vec![],
        }
    }
}