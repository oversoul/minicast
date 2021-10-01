#![allow(dead_code)]
#![warn(unused_imports)]

use std::{error::Error, io};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Gauge, List, ListItem},
    Terminal,
};

pub struct FeedView<'a> {
    name: &'a str,
    focus: bool,
    data: &'a [&'a str],
}

impl<'a> FeedView<'a> {
    pub fn new(name: &'a str, data: &'a [&'a str]) -> Self {
        FeedView {
            name,
            data,
            focus: true,
        }
    }

    pub fn set_data(&mut self, data: &'a [&'a str]) {
        self.data = data;
    }

    pub fn draw(&self) -> List {
        // Create a List from all list items and highlight the currently selected one
        let items: Vec<ListItem> = self.data.iter().map(|x| ListItem::new(*x)).collect();

        return List::new(items)
            .block(Block::default().title("Feeds").borders(Borders::ALL))
            .highlight_style(self.get_style())
            .highlight_symbol(">> ");
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
        let items: Vec<ListItem> = self.data.iter().map(|x| ListItem::new(x.as_str())).collect();

        return List::new(items)
            .block(Block::default().title("Episodes").borders(Borders::ALL))
            .highlight_style(self.get_style())
            .highlight_symbol(">> ");
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

