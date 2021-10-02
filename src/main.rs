#![allow(dead_code)]
#![warn(unused_imports)]
extern crate tui;

mod app;
mod events;
mod feed;
mod player;
mod state;
mod ui;

use player::MediaWorker;
use state::State;
use ui::{EpisodeView, FeedView};

use events::{Event, Events};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{backend::TermionBackend, Terminal};

fn main() {
    let url = "https://feeds.simplecast.com/sY509q85";
    // let url = "https://feeds.megaphone.fm/stuffyoushouldknow";
    // let url = "https://www.ted.com/feeds/talks.rss";

    //let url = "https://rss.art19.com/smartless";

    // Terminal initialization
    let stdout = std::io::stdout().into_raw_mode().expect("std out");
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend).expect("terminal...");

    let events = Events::new();
    let mut app = app::App::new();
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
                let meta_data = match app.get_current_episode_meta() {
                    (title, description) => ui::meta_data(title, description),
                };

                let (top, bottom) = ui::main_layout(f.size());
                let (left, middle, right) = ui::top_sections(top);
                let player = ui::player_progress(media.time_position().unwrap_or("".into()));

                f.render_stateful_widget(feed_view.draw(), left, &mut feed_state.value);
                f.render_stateful_widget(episode_view.draw(), middle, &mut episode_state.value);
                f.render_widget(meta_data, right);
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
                    media.stop().expect("Couldn't quit.");
                }
                Key::Char('\n') => {
                    if feed_view.in_focus() {
                        app.load_feed(url);

                        let episodes = app.get_episodes_title();
                        let eps_len = &episodes.len();

                        feed_view.set_focus(false);
                        episode_view.set_focus(true);
                        episode_state.set_max(*eps_len);
                        episode_view.set_data(episodes);
                    } else {
                        let ep = episode_state.get_value();
                        let episode = app.get_episode_by_index(ep);
                        app.set_playing_episode_meta(episode.title, episode.description);

                        let url = episode.url.as_str();
                        media.loadfile(url).unwrap();
                    }
                }
                _ => {}
            },
            Ok(Event::Tick) => {}
            Err(_) => {
                break;
            }
        }
    }
}
