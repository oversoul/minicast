#![allow(dead_code)]
#![warn(unused_imports)]

use tui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text,
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Wrap},
};

pub struct FeedView {
    focus: bool,
    data: Vec<String>,
}

impl FeedView {
    pub fn new(data: Vec<String>) -> Self {
        FeedView { data, focus: true }
    }

    pub fn set_data(&mut self, data: Vec<String>) {
        self.data = data;
    }

    pub fn draw(&self) -> List {
        let items: Vec<ListItem> = self
            .data
            .iter()
            .map(|x| ListItem::new(x.as_str()))
            .collect();

        List::new(items)
            .block(Block::default().title("Feeds").borders(Borders::ALL))
            .highlight_style(self.get_style())
    }

    pub fn in_focus(&mut self) -> bool {
        self.focus
    }

    fn get_style(&self) -> Style {
        let color = if self.focus {
            Color::LightGreen
        } else {
            Color::Gray
        };

        Style::default().bg(color).add_modifier(Modifier::BOLD)
    }

    pub fn set_focus(&mut self, focus: bool) {
        self.focus = focus;
    }
}

pub struct EpisodeView {
    focus: bool,
    data: Vec<String>,
}

impl EpisodeView {
    pub fn new(data: Vec<String>) -> Self {
        EpisodeView { data, focus: false }
    }

    pub fn set_data(&mut self, data: Vec<String>) {
        self.data = data;
    }

    pub fn in_focus(&mut self) -> bool {
        self.focus
    }

    pub fn draw(&self) -> List {
        // Create a List from all list items and highlight the currently selected one
        let items: Vec<ListItem> = self
            .data
            .iter()
            .map(|x| ListItem::new(x.as_str()))
            .collect();

        List::new(items)
            .block(Block::default().title("Episodes").borders(Borders::ALL))
            .highlight_style(self.get_style())
    }

    fn get_style(&self) -> Style {
        let color = if self.focus {
            Color::LightGreen
        } else {
            Color::Gray
        };

        Style::default().bg(color).add_modifier(Modifier::BOLD)
    }

    pub fn set_focus(&mut self, focus: bool) {
        self.focus = focus;
    }
}

pub fn main_layout(size: Rect) -> (Rect, Rect) {
    let main_view = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
        .split(size);
    (main_view[0], main_view[1])
}

pub fn top_sections(size: Rect) -> (Rect, Rect, Rect) {
    let sections = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ]
            .as_ref(),
        )
        .split(size);
    (sections[0], sections[1], sections[2])
}

pub fn player<'a>(ratio: f64) -> Gauge<'a> {
    Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Progress"))
        .gauge_style(Style::default().fg(Color::Blue).bg(Color::Black))
        .ratio(ratio)
}

pub fn meta_data<'a, T: Into<String>>(title: T, description: T) -> Paragraph<'a> {
    let title = title.into();
    let description = description.into();
    let block = Block::default().title("Metadata").borders(Borders::ALL);

    if title.len() == 0 {
        return Paragraph::new(vec![]).block(block);
    }

    let bold_style = Style::default().add_modifier(Modifier::BOLD);

    let text = vec![
        text::Spans::from(text::Span::styled("Title:", bold_style)),
        text::Spans::from(title),
        text::Spans::from(vec![]),
        text::Spans::from(text::Span::styled("Description:", bold_style)),
        text::Spans::from(description),
    ];

    Paragraph::new(text)
        .block(block)
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true })
}

pub fn player_progress<'a, T: Into<String>>(position: T) -> Paragraph<'a> {
    let text = vec![text::Spans::from(position.into())];

    Paragraph::new(text)
        .block(Block::default().title("Player").borders(Borders::ALL))
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true })
}
