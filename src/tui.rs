use crate::api::HNItem;
use anyhow::Result;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::Widget;
use ratatui::style::palette::tailwind::SLATE;
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::symbols::border;
use ratatui::text::Line;
use ratatui::widgets::StatefulWidget;
use ratatui::widgets::{Block, HighlightSpacing, List, ListItem, ListState, Paragraph};
use ratatui::DefaultTerminal;
const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);

#[derive(Debug, Default)]
enum Mode {
    #[default]
    Story,
    Detail
}

#[derive(Debug, Default)]
pub struct Detail {
    pub active_story: Option<HNItem>
}

#[derive(Debug, Default)]
pub struct App {
    stories: Vec<HNItem>,
    stories_state: ListState,
    detail: Detail,
    exit: bool,
    mode: Mode
}

impl App {
    pub fn new() -> Self {
        let mut ls = ListState::default();
        ls.select(Some(0));
        Self {
            stories: mock_hnstories(),
            stories_state: ls,
            detail: Default::default(),
            exit: false,
            mode: Default::default(),
        }
    }

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            self.handle_events()?;
        }

        Ok(())
    }

    fn handle_events(&mut self) -> Result<()>{
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event);
            }
            _ => {}
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    pub fn render_story_list(&mut self, area: Rect, buf: &mut Buffer) {
        let main_block = Block::bordered()
            .title(" Stories / Comments ")
            .border_set(border::PLAIN);

        let stories: Vec<_> = self
            .stories
            .iter()
            .map(|item| ListItem::from(item))
            .collect();

        let list = List::new(stories)
            .block(main_block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut self.stories_state);
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized
    {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // header
                Constraint::Min(1),     // main scrollable content
                Constraint::Length(1),  // footer
            ])
            .split(area);

        let title = Line::from(" HN Stories ".bold());
        let header_block = Block::bordered()
            .title(title.centered())
            .border_set(border::THICK);

        Paragraph::new("")
            .block(header_block)
            .render(chunks[0], buf);

        self.render_story_list(chunks[1], buf);

        let footer_text = Line::from(" j/k: move  Enter: select  Esc: back  q: quit ");
        Paragraph::new(footer_text.centered())
            .render(chunks[2], buf);
    }
}

impl From<&HNItem> for ListItem<'_> {
    fn from(value: &HNItem) -> Self {
        let line = match &value.by {
            Some(nickname) => Line::from(format!("Written by {} with id {}", nickname, value.id)),
            None => Line::from(format!("Written by anonymous with id {}", value.id)),
        };

        ListItem::new(line)
    }
}

fn mock_hnstories() -> Vec<HNItem> {
    vec![HNItem{
        id: 0,
        by: Some("guy".parse().unwrap()),
        title: None,
        url: None,
        kids: None,
        score: None,
        time: None,
        text: None,
        r#type: None,
    }, HNItem{
        id: 1,
        by: None,
        title: None,
        url: None,
        kids: None,
        score: None,
        time: None,
        text: None,
        r#type: None,
    }]
}