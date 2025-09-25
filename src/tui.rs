use crate::api::{HNApi, HNItem};
use anyhow::Result;
use color_eyre::owo_colors::OwoColorize;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::Widget;
use ratatui::style::palette::tailwind::SLATE;
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::symbols::border;
use ratatui::text::{Line, Text, Span};
use ratatui::widgets::{StatefulWidget, Wrap};
use ratatui::widgets::{Block, HighlightSpacing, List, ListItem, ListState, Paragraph};
use ratatui::DefaultTerminal;
const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);

struct Theme {
    bg: Color,
    story_title: Color,
    selected_bg: Color,
    author: Color,
    comment_text: Color,
    collapsed_marker: Color,
    footer: Color,
}
#[derive(Debug, Default)]
enum Mode {
    #[default]
    Story,
    Detail
}

#[derive(Debug, Default)]
pub struct App {
    stories: Vec<HNItem>,
    client: HNApi,
    stories_state: ListState,
    detail: Option<usize>,
    exit: bool,
    mode: Mode
}

enum ScrollDirection {
    Up,
    Down,
}

impl ScrollDirection {
    fn from_char(c: char) -> Option<Self> {
        match c {
            'j' => Some(ScrollDirection::Up),
            'k' => Some(ScrollDirection::Down),
            _ => None
        }
    }
}

impl App {
    pub fn new(client: HNApi, top_stories: Vec<HNItem>) -> Self {
        let mut ls = ListState::default();
        ls.select(Some(0));
        //just for test
        Self {
            stories: top_stories,
            client,
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
            KeyCode::Char(c) if c == 'j' || c == 'k' => {
                if let Some(direction) = ScrollDirection::from_char(c) {
                    self.scroll(direction);
                }
            }
            KeyCode::Enter => self.show_story(),
            KeyCode::Esc => self.back_to_list(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn show_story(&mut self) {
        if let Some(selected) = self.stories_state.selected() {
            self.detail = Some(selected);
            self.mode = Mode::Detail;
        }
    }

    fn scroll(&mut self, direction: ScrollDirection) {
        match direction {
            ScrollDirection::Up => self.stories_state.scroll_up_by(1),
            ScrollDirection::Down =>self.stories_state.scroll_down_by(1),
        }
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

    pub fn render_story_details(&mut self, area: Rect, buf: &mut Buffer) {
        let main_block = Block::bordered()
            .title(" Story details ")
            .border_set(border::PLAIN);

        let text = if let Some(idx) = self.detail {
            if let Some(item) = self.stories.get(idx) {
                let mut lines: Vec<Line> = Vec::new();

                let title_str = item.title.as_deref().unwrap_or("<no title>");
                lines.push(Line::from(vec![
                    Span::styled(title_str, Style::default().add_modifier(Modifier::BOLD)),
                ]));
                lines.push(Line::from(format!("by: {}", item.by.as_deref().unwrap_or("<unknown>"))));

                let url_str = item.url.as_deref().unwrap_or("N/A");
                lines.push(Line::from(format!("url: {}", url_str)));
                lines.push(Line::from(""));

                if let Some(body) = &item.text {
                    for raw_line in body.lines() {
                        lines.push(Line::from(raw_line.to_string()));
                    }
                } else {
                    lines.push(Line::from("<no body>"));
                }

                Text::from(lines)
            } else {
                Text::from("Selected story no longer exists")
            }
        } else {
            Text::from("No story selected")
        };

        let paragraph = Paragraph::new(text)
            .block(main_block)
            .wrap(Wrap { trim: true });

        paragraph.render(area, buf);
    }

    fn back_to_list(&mut self) {
        self.mode = Mode::Story;
        self.detail = None;
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
            .border_set(border::THICK)
            .style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Rgb(255,140,0))
            );

        Paragraph::new("")
            .block(header_block)
            .render(chunks[0], buf);

        match self.mode {
            Mode::Story => self.render_story_list(chunks[1], buf),
            Mode::Detail => self.render_story_details(chunks[1], buf),
        }

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