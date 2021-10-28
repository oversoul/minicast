#![allow(dead_code)]
#![warn(unused_imports)]
extern crate cursive;

mod app;
mod db;
// mod events;
mod feed;
mod player;
// mod state;
// mod ui;

use player::MediaWorker;
// use state::State;
// use ui::{EpisodeView, FeedView};

use cursive::{
    align::HAlign,
    event::{EventResult, Key},
    traits::With,
    traits::*,
    view::{scroll::Scroller, Scrollable, SizeConstraint},
    views::{
        Button, Dialog, DummyView, FixedLayout, LinearLayout, ListView, OnEventView, Panel,
        ProgressBar, ResizedView, SelectView, TextArea, TextContent, TextView,
    },
    Cursive, Rect,
};
use std::sync::{Arc, Mutex};

fn main() {
    let app = app::App::new();
    let media = MediaWorker::new().expect("Couldn't open media worker");

    let app = Arc::new(Mutex::new(app));
    let media = Arc::new(Mutex::new(media));

    let mut siv = cursive::default();

    let mut feeds_select = SelectView::new().autojump();

    let mut episodes_select: SelectView<u32> = SelectView::new();

    let app_ref = Arc::clone(&app);
    feeds_select.add_all(app_ref.lock().unwrap().get_feeds_name_id());

    feeds_select.set_on_submit(move |s, p: &u32| {
        let ep_id = *p;
        let app_ref = Arc::clone(&app_ref);
        let new_app = app_ref.lock().unwrap();
        let mut episodes = s.find_name::<SelectView<_>>("episodes").unwrap();

        new_app.reload_episodes(ep_id);

        let eps = new_app.get_episodes_title_id(ep_id);
        episodes.clear();
        episodes.add_all(eps);
        s.focus_name("episodes").unwrap();
    });

    let app_ref = Arc::clone(&app);
    let media_ref = Arc::clone(&media);
    episodes_select.set_on_submit(move |s, id: &u32| {
        let new_app = app_ref.lock().unwrap();
        let episode = new_app.get_episode(*id);

        if let Some(ref mut view) = s.find_name::<TextView>("ep_title") {
            view.set_content(episode.title)
        };

        if let Some(ref mut view) = s.find_name::<TextView>("ep_description") {
            view.set_content(episode.description)
        };

        media_ref.lock().unwrap().loadfile(&episode.url).unwrap();
    });

    let media_ref = Arc::clone(&media);
    // siv.on_event(cursive::event::Event::Refresh);

    siv.add_fullscreen_layer(
        LinearLayout::vertical()
            .child(
                LinearLayout::horizontal()
                    .child(
                        Panel::new(feeds_select.with_name("feed").scrollable())
                            .title("Feed")
                            .full_width()
                            .full_height(),
                    )
                    .child(
                        Panel::new(episodes_select.with_name("episodes").scrollable())
                            .title("Episodes")
                            .full_width()
                            .full_height(),
                    )
                    .child(
                        Panel::new(
                            LinearLayout::vertical()
                                .child(TextView::new("Name:"))
                                .child(TextView::new("").with_name("ep_title"))
                                .child(TextView::new("\nDescription:"))
                                .child(TextView::new("").with_name("ep_description")),
                        )
                        .title("Details")
                        .full_width()
                        .full_height(),
                    ),
            )
            .child(
                Panel::new(
                    ProgressBar::new()
                        .range(0, 100)
                        .with_label(move |v, (b, e)| {
                            let media = media_ref.lock().unwrap();
                            match media.percentage() {
                                Ok(p) => format!("{} %", p),
                                _ => "".into(),
                            }
                            // format!("{} %", value)
                        })
                        .full_width(),
                )
                .title("Player"),
            ),
    );

    siv.add_global_callback('q', |s| s.quit());
    siv.run();
    // loop {}

    /*
        let events = Events::new();
        let mut app = app::App::new();

        let mut media = MediaWorker::new().expect("Couldn't open media worker");

        let feeds = app.get_feeds_name();
        let mut feed_state = State::new();
        feed_state.set_max(feeds.len());

        let mut episode_state = State::new();
        let mut feed_view = FeedView::new(feeds);
        let mut episode_view = EpisodeView::new(vec![]);

        let mut show_popup = false;
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

                    if show_popup {
                        let block = Block::default().title("Add Feed").borders(Borders::ALL);
                        let area = ui::centered_rect(60, 20, f.size());
                        f.render_widget(Clear, area); //this clears out the background
                        f.render_widget(block, area);
                    }
                })
                .expect("terminal loop...");

            match events.next() {
                Ok(Event::Input(input)) => match input {
                    Key::Char('q') => break,
                    Key::Down => {
                        if episode_view.in_focus() {
                            episode_state.increment();
                        } else if feed_view.in_focus() {
                            feed_state.increment();
                        }
                    }
                    Key::Left => {
                        if !feed_view.in_focus() {
                            episode_view.set_focus(false);
                            feed_view.set_focus(true);
                        }
                    }
                    Key::Right => {
                        if !episode_view.in_focus() {
                            feed_view.set_focus(false);
                            episode_view.set_focus(true);
                        }
                    }
                    Key::Up => {
                        if episode_view.in_focus() {
                            episode_state.decrement();
                        } else if feed_view.in_focus() {
                            feed_state.decrement();
                        }
                    }
                    Key::Char('a') => {
                        show_popup = true;
                    }
                    Key::Esc => {
                        show_popup = false;
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
                            let episodes = app.get_episodes_title(feed.unwrap());
                            episode_state.set_max(episodes.len());

                            feed_view.set_focus(false);
                            episode_view.set_focus(true);
                            episode_view.set_data(episodes);
                        }
                    }
                    Key::Char('\n') => {
                        if feed_view.in_focus() {
                            feed = Some(app.get_feed_id(feed_state.get_value()));

                            let episodes = app.get_episodes_title(feed.unwrap());
                            episode_state.reset();
                            episode_state.set_max(episodes.len());

                            feed_view.set_focus(false);
                            episode_view.set_focus(true);
                            episode_view.set_data(episodes);
                        } else {
                            let ep_id = app.get_episode_id(feed.unwrap(), episode_state.get_value());

                            let episode = app.get_episode(ep_id);

                            title = episode.title.to_owned();
                            description = episode.description.to_owned();
                            app.set_playing_episode_meta(episode.title, episode.description);

                            media.loadfile(&episode.url).unwrap();
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
    */
}
