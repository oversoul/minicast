use crate::db::{Database, Episode};
use crate::feed;
use crate::feed::Feed;

pub struct App<'a> {
    pub feed: Option<&'a Feed>,
    feeds: Vec<Feed>,
    episode_title: String,
    episode_description: String,
    db: Database,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        App {
            feed: None,
            feeds: vec![],
            episode_title: "".into(),
            episode_description: "".into(),
            db: Database::new().expect("wrong"),
        }
    }

    pub fn get_feed_id(&mut self, idx: usize) -> u32 {
        let feed = &self.db.get_feeds()[idx];
        feed.id
    }

    pub fn get_episode_id(&mut self, feed_id: u32, idx: usize) -> u32 {
        let episodes: Vec<Episode> = self.db.get_episodes(feed_id);
        episodes[idx].id
    }

    pub fn reload_episodes(&self, feed_id: u32) {
        let feed = self.db.get_feed(feed_id);

        self.db.clear_episodes(feed_id).unwrap();

        let episodes = match feed {
            Ok(f) => feed::get_episodes(Feed::Url(f.url)),
            _ => vec![],
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

    pub fn get_current_episode_meta(&self) -> (&str, &str) {
        (&self.episode_title, &self.episode_description)
    }
}
