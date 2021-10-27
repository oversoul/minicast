#![allow(dead_code)]
#![warn(unused_imports)]
extern crate tui;

mod app;
mod db;
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
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{backend::TermionBackend, Terminal};

fn main() {
    // Terminal initialization
    let stdout = std::io::stdout().into_raw_mode().expect("std out");
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend).expect("terminal...");

    let events = Events::new();
    let mut app = app::App::new();

    let mut media = MediaWorker::new().expect("Couldn't open media worker");

    let feeds_len = app.get_feeds_name().len();
    let mut feed_state = State::new();
    feed_state.set_max(feeds_len);

    let mut episode_state = State::new();
    let mut feed_view = FeedView::new(app.get_feeds_name());

    let mut episode_view = EpisodeView::new(vec![]);

    let mut feed: Option<u32> = None;
    let mut title = String::from("");
    let mut description = String::from("");

    loop {
        terminal
            .draw(|f| {
                let meta_data = ui::meta_data(title.as_str(), description.as_str());

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
                Key::Char('r') => {
                    if episode_view.in_focus() && feed.is_some() {
                        app.reload_episodes(feed.unwrap());
                        // reload view...
                    }
                }
                Key::Char('\n') => {
                    if feed_view.in_focus() {
                        feed = Some(app.get_feed_id(feed_state.get_value()));

                        let episodes = app.get_episodes_title(feed.unwrap());
                        let eps_len = episodes.len();

                        feed_view.set_focus(false);
                        episode_view.set_focus(true);
                        episode_state.set_max(eps_len);
                        episode_view.set_data(episodes);
                    } else {
                        let ep = episode_state.get_value();

                        // let episode = feed.as_ref().unwrap().get_episode_by_index(ep);
                        // title = episode.title.to_owned();
                        // description = episode.description.to_owned();
                        //app.set_playing_episode_meta(episode.title, episode.description);

                        // let url = episode.url.as_str();
                        // media.loadfile(url).unwrap();
                    }
                }
                _ => {}
            },
            Ok(Event::Tick) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }
}
