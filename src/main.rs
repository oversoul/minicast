#![allow(dead_code)]
mod feed;
use feed::Feed;
// use std::path::Path;

fn main() {
    // let url = "https://feeds.simplecast.com/sY509q85";
    // let url = "https://feeds.megaphone.fm/stuffyoushouldknow";
    // let url = "https://www.ted.com/feeds/talks.rss";
    let url = "https://rss.art19.com/smartless";
    let mut feed = Feed::from_url(url);

    feed.parse_episodes();
    for episode in feed.episodes {
        println!("{}", episode.title);
    }
}
