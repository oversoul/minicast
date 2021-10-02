#![allow(dead_code)]
#![warn(unused_imports)]
extern crate tui;

mod events;
mod feed;
mod player;
mod state;
mod ui;

use feed::Feed;
use player::MediaWorker;
use state::State;
use ui::{EpisodeView, FeedView};

use events::{Event, Events};
use std::{error::Error, io};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    widgets::{Block, Borders, Gauge, LineGauge, List, ListItem},
    Terminal,
};


fn main() {
    let url = "https://feeds.simplecast.com/sY509q85";
    // let url = "https://feeds.megaphone.fm/stuffyoushouldknow";
    // let url = "https://www.ted.com/feeds/talks.rss";

    //let url = "https://rss.art19.com/smartless";

    // Terminal initialization
    let stdout = io::stdout().into_raw_mode().expect("std out");
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend).expect("terminal...");

    let events = Events::new();

    // let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    // let mut player = Player::new(&stream_handle);
    let mut media = MediaWorker::new().expect("Couldn't open media worker");

    let list_items = &["Laracasts"];

    let mut feed_state = State::new();
    feed_state.set_max(list_items.len());

    let mut episode_state = State::new();

    let mut feed_view = FeedView::new(url, list_items);
    let mut episode_view = EpisodeView::new(vec![]);

    loop {
        terminal
            .draw(|f| {
                let (top, bottom) = ui::main_layout(f.size());
                let (left, middle, right) = ui::top_sections(top);
                let block = Block::default().title("Metadata").borders(Borders::ALL);
                let player = ui::player(media.percentage().unwrap_or(0.0) / 100.0);

                f.render_stateful_widget(feed_view.draw(), left, &mut feed_state.value);
                f.render_stateful_widget(episode_view.draw(), middle, &mut episode_state.value);
                f.render_widget(block, right);
                f.render_widget(player, bottom);
            })
            .expect("terminal loop...");

        match events.next() {
            Ok(Event::Input(input)) => match input {
                Key::Char('q') => {
                    break;
                }
                Key::Down => {
                    if episode_view.in_focus() {
                        episode_state.increment();
                    } else if feed_view.in_focus() {
                        feed_state.increment();
                    }
                }
                Key::Left => {
                    if feed_view.in_focus() {
                        return;
                    }
                    episode_view.set_focus(false);
                    feed_view.set_focus(true);
                }
                Key::Right => {
                    if episode_view.in_focus() {
                        return;
                    }
                    feed_view.set_focus(false);
                    episode_view.set_focus(true);
                }
                Key::Up => {
                    if episode_view.in_focus() {
                        episode_state.decrement();
                    } else if feed_view.in_focus() {
                        feed_state.decrement();
                    }
                }
                Key::Char('p') => {
                    media.toggle_play().expect("toggle play issue");
                }
                Key::Char('s') => {
                    media.stop().expect("Couldn't stop.");
                }
                Key::Char('\n') => {
                    if feed_view.in_focus() {
                        let mut feed = Feed::from_url(url);

                        feed.parse_episodes();

                        let ep_len = feed.episodes.len();
                        let episodes = feed
                            .episodes
                            .iter()
                            .map(|e| format!("{}", e.title))
                            .collect();

                        feed_view.set_focus(false);
                        episode_view.set_focus(true);
                        episode_view.set_data(episodes);
                        episode_state.set_max(ep_len);
                    } else {
                        media.stop().expect("couldn't stop");
                        let ep = episode_state.get_value();

                        // temporary
                        let mut feed = Feed::from_url(url);

                        feed.parse_episodes();

                        let url = feed.episodes[ep].url.as_str();
                        media.loadfile(url).unwrap();
                    }
                }
                _ => {}
            },
            Ok(Event::Tick) => {}
            Err(_) => {}
        }
    }
}
