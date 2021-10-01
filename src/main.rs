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
    widgets::{Block, Borders, Gauge, List, ListItem},
    Terminal,
};

// rodio
use rodio::source::{SineWave, Source};
use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;
use std::time::Duration;

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
    let mut episodes: Vec<String> = vec![];
    let mut episode_view = EpisodeView::new(episodes);

    loop {
        terminal
            .draw(|f| {
                let (top, bottom) = ui::main_layout(f.size());
                let (left, middle, right) = ui::top_sections(top);

                f.render_stateful_widget(feed_view.draw(), left, &mut feed_state.value);
                f.render_stateful_widget(episode_view.draw(), middle, &mut episode_state.value);

                let block = Block::default().title("Metadata").borders(Borders::ALL);

                let player = Gauge::default()
                    .block(Block::default().borders(Borders::ALL).title("Progress"))
                    .gauge_style(Style::default().fg(Color::Blue).bg(Color::Black))
                    .ratio(media.percentage().unwrap_or(0.0) / 100.0);

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

                    /*
                    let ep_url = "https://cdn.simplecast.com/audio/fd7dca0e-5e82-4d04-b65f-c0aa44661798/episodes/fd0bd2ba-c553-466c-a060-b144797ce369/audio/4c6c7b70-68b7-41fe-8280-af8a9d476b0a/default_tc.mp3?aid=embed";
                    media.loadfile(ep_url).unwrap();
                    */
                    // let state_value = feed_state.get_value();
                    // player.play_url();
                }
                _ => {}
            },
            Ok(Event::Tick) => {}
            Err(_) => {}
        }
    }
}
