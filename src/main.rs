#![allow(dead_code)]
mod feed;
use feed::Feed;

use cursive::align::HAlign;
use cursive::view::Scrollable;
use cursive::views::{
    Button, Dialog, DummyView, EditView, FixedLayout, LinearLayout, ListView, SelectView, TextView,
};
use cursive::Cursive;
use cursive::{Rect, With};
use rodio::{source::Source, Decoder, OutputStream, Sink};
use std::io::{Cursor, Read, Write};
use std::sync::mpsc;
use std::sync::Arc;

fn main() {
    let url = "https://feeds.simplecast.com/sY509q85";
    // let url = "https://feeds.megaphone.fm/stuffyoushouldknow";
    // let url = "https://www.ted.com/feeds/talks.rss";

    //let url = "https://rss.art19.com/smartless";
    let mut feed = Feed::from_url(url);

    feed.parse_episodes();

    let mut siv = cursive::default();

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Arc::new(Sink::try_new(&stream_handle).unwrap());

    siv.add_global_callback('q', |s| s.quit());

    let mut feeds_list = SelectView::new()
        // Center the text horizontally
        .h_align(HAlign::Left)
        // Use keyboard to jump to the pressed letters
        .autojump();

    let new_sink = Arc::clone(&sink);
    feeds_list.set_on_submit(move |s, e| {
        play_episode(s, e, &sink);
    });

    for episode in feed.episodes {
        feeds_list.add_item(&episode.title, episode.url);
    }

    let buttons = LinearLayout::horizontal()
        .child(feeds_list.scrollable())
        .child(Button::new("Delete", Cursive::quit))
        .child(Button::new("Quit", Cursive::quit));

    siv.add_layer(buttons);

    new_sink.sleep_until_end();

    siv.run();
}

fn play_episode(siv: &mut Cursive, url: &String, sink: &Arc<Sink>) {
    // siv.pop_layer();
    // let (tx, rx) = mpsc::channel();
    let new_sink = Arc::clone(&sink);

    siv.add_layer(
        Dialog::around(TextView::new(url))
            .button("Stop", move |s| {
                new_sink.stop();
                s.pop_layer();
            })
            .button("Quit", |s| s.quit()),
    );

    let ep_url = url.clone();
    let new_sink = Arc::clone(&sink);

    std::thread::spawn(move || {
        let resp = minreq::get(ep_url).send().expect("not working");
        let output = resp.into_bytes();

        let cursor = Cursor::new(output); // Adds Read and Seek to the bytes via Cursor
        let source = rodio::Decoder::new(cursor).unwrap();

        new_sink.append(source);
    });
    // sink.sleep_until_end();
}
