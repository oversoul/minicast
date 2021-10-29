use crate::app;
use crate::player::MediaWorker;
use crate::ui::{Ui, UiMessage};
use std::sync::mpsc;

pub struct Controller {
    rx: mpsc::Receiver<ControllerMessage>,
    ui: Ui,
    app: app::App,
    media: MediaWorker,
}

pub enum ControllerMessage {
    LoadFeeds,
    UpdatePlayState,
    UpdateStopPlayer,

    DeleteFeed(u32),
    AddNewFeed(String, String),
    UpdatePlayEpisode(u32),
    UpdateSelectedFeed(u32),
    ReloadFeedEpisodes(u32),
}

impl Controller {
    pub fn new() -> Result<Controller, String> {
        let (tx, rx) = mpsc::channel::<ControllerMessage>();
        Ok(Controller {
            rx: rx,
            app: app::App::new(),
            media: MediaWorker::new().expect("can't open media"),
            ui: Ui::new(tx.clone()),
        })
    }

    pub fn run(&mut self) {
        while self.ui.step() {
            if !self.media.is_paused {
                self.ui
                    .ui_tx
                    .send(UiMessage::UpdateProgress(self.media.percent().unwrap_or(0)))
                    .unwrap();
            }
            while let Some(message) = self.rx.try_iter().next() {
                match message {
                    ControllerMessage::LoadFeeds => {
                        let feeds = self.app.get_feeds_name_id();
                        self.ui.ui_tx.send(UiMessage::UpdateFeeds(feeds)).unwrap();
                    }
                    ControllerMessage::UpdatePlayEpisode(episode) => {
                        let episode = self.app.get_episode(episode);
                        self.ui
                            .ui_tx
                            .send(UiMessage::UpdatePlaying(episode.title, episode.description))
                            .unwrap();

                        self.media.loadfile(&episode.url).unwrap();
                    }
                    ControllerMessage::ReloadFeedEpisodes(feed) => {
                        self.app.reload_episodes(feed);
                        let eps = self.app.get_episodes_title_id(feed);
                        self.ui.ui_tx.send(UiMessage::UpdateEpisodes(eps)).unwrap();
                    }
                    ControllerMessage::UpdatePlayState => {
                        self.media.toggle_play().expect("can't toggle play state");
                    }
                    ControllerMessage::UpdateStopPlayer => {
                        self.media.stop().expect("can't toggle play state");
                    }
                    ControllerMessage::UpdateSelectedFeed(feed) => {
                        let eps = self.app.get_episodes_title_id(feed);
                        self.ui.ui_tx.send(UiMessage::UpdateEpisodes(eps)).unwrap();
                    }
                    ControllerMessage::AddNewFeed(name, url) => {
                        self.app.add_feed(name, url);
                        let feeds = self.app.get_feeds_name_id();
                        self.ui.ui_tx.send(UiMessage::UpdateFeeds(feeds)).unwrap();
                    }
                    ControllerMessage::DeleteFeed(id) => {
                        self.app.delete_feed(id);
                        let feeds = self.app.get_feeds_name_id();
                        self.ui.ui_tx.send(UiMessage::UpdateFeeds(feeds)).unwrap();
                    }
                };
            }
        }
    }
}
