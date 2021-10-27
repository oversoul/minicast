use crate::db::{Database, Episode};
use crate::feed;
use crate::feed::Feed;

pub struct App {
    db: Database,
    episode_title: String,
    episode_description: String,
}

impl App {
    pub fn new() -> Self {
        App {
            episode_title: "".into(),
            episode_description: "".into(),
            db: Database::new().expect("wrong"),
        }
    }

    pub fn get_feed_id(&self, idx: usize) -> u32 {
        let feeds = self.db.get_feeds();
        feeds[idx].id
    }

    pub fn get_episode_id(&mut self, feed_id: u32, idx: usize) -> u32 {
        let episodes = self.db.get_episodes(feed_id);
        episodes[idx].id
    }

    pub fn reload_episodes(&self, feed_id: u32) {
        let feed = self.db.get_feed(feed_id);

        self.db.clear_episodes(feed_id).unwrap();

        let episodes = match feed {
            Ok(f) => feed::get_episodes(Feed::Url(f.url)),
            Err(_) => vec![],
        };

        let episodes = episodes
            .into_iter()
            .map(|e| Episode {
                id: 0,
                url: e.url,
                title: e.title,
                description: e.description,
            })
            .collect();

        self.db.set_episodes(feed_id, episodes).unwrap();
    }

    pub fn get_episodes_title(&self, feed: u32) -> Vec<String> {
        self.db
            .get_episodes(feed)
            .into_iter()
            .map(|e| e.title)
            .collect()
    }

    pub fn get_feeds_name(&self) -> Vec<String> {
        self.db.get_feeds().into_iter().map(|e| e.name).collect()
    }

    pub fn get_episode(&self, id: u32) -> Episode {
        self.db.get_episode(id).unwrap()
    }

    pub fn set_playing_episode_meta(&mut self, title: String, description: String) {
        self.episode_title = title;
        self.episode_description = description;
    }
}
