use crate::feed::{Episode, Feed};

pub struct App {
    feed: Option<Feed>,
    episode_title: String,
    episode_description: String,
}

impl App {
    pub fn new() -> Self {
        App {
            feed: None,
            episode_title: "".into(),
            episode_description: "".into(),
        }
    }

    pub fn load_feed(&mut self, url: &str) {
        let mut feed = Feed::from_url(url);
        feed.parse_episodes();
        self.set_feed(feed);
    }

    pub fn set_feed(&mut self, feed: Feed) {
        self.feed = Some(feed);
    }

    pub fn get_episodes_title(&self) -> Vec<String> {
        self.feed
            .as_ref()
            .unwrap()
            .episodes
            .iter()
            .map(|e| format!("{}", e.title))
            .collect()
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
