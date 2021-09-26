#![allow(dead_code)]
mod feed;
use feed::Feed;
use std::path::Path;
// laracasts
// //
fn main() {
    let url = "https://feeds.simplecast.com/sY509q85";
    let mut feed = Feed::from_url(url);

    feed.parse_episodes();
    for episode in feed.episodes {
        println!("{} => {:?}", episode.description, episode.enclosure.unwrap().url);
    }
}
