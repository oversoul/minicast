use crate::feed::{Episode, Feed};

pub struct App<'a> {
    pub feed: Option<&'a Feed>,
    feeds: Vec<Feed>,
    episode_title: String,
    episode_description: String,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        App {
            feed: None,
            feeds: vec![],
            episode_title: "".into(),
            episode_description: "".into(),
        }
    }

    pub fn set_feeds(&mut self, feeds: Vec<Feed>) {
        self.feeds = feeds;
    }

    pub fn get_feed_by_idx(&mut self, idx: usize) -> Feed {
        let feed = &mut self.feeds[idx];
        Feed {
            name: feed.name.clone(),
            url: feed.url.clone(),
            is_url: feed.is_url,
            path: feed.path.clone(),
            channel: feed.channel.clone(),
            episodes: feed.parse_episodes(),
        }
    }

    pub fn get_feeds_name(&self) -> Vec<&str> {
        self.feeds.iter().map(|e| e.name.as_str()).collect()
    }

    pub fn get_episode_by_index(&self, idx: usize) -> Episode {
        let ep = &self.feed.as_ref().unwrap().episodes[idx];
        Episode::new_from_ref(ep)
    }

    pub fn set_playing_episode_meta(&mut self, title: String, description: String) {
        self.episode_title = title;
        self.episode_description = description;
    }

    pub fn get_current_episode_meta(&self) -> (&str, &str) {
        (&self.episode_title, &self.episode_description)
    }
}
