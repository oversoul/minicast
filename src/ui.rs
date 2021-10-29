use crate::controller::ControllerMessage;
//use cursive::view::*;
use cursive::event::Key;
use cursive::view::{Nameable, Resizable, Scrollable, SizeConstraint};
use cursive::views::*;
use std::sync::mpsc;

pub struct Ui {
    ui_rx: mpsc::Receiver<UiMessage>,
    pub ui_tx: mpsc::Sender<UiMessage>,
    cursive: cursive::CursiveRunner<cursive::CursiveRunnable>,
    controller_tx: mpsc::Sender<ControllerMessage>,
}

pub enum UiMessage {
    UpdateProgress(usize),
    UpdatePlaying(String, String),
    UpdateFeeds(Vec<(String, u32)>),
    UpdateEpisodes(Vec<(String, u32)>),
}

impl Ui {
    /// Create a new Ui object.  The provided `mpsc` sender will be used
    /// by the UI to send messages to the controller.
    pub fn new(controller_tx: mpsc::Sender<ControllerMessage>) -> Ui {
        let (ui_tx, ui_rx) = mpsc::channel::<UiMessage>();
        let siv = cursive::CursiveRunnable::default();
        let cursive = siv.into_runner();

        let mut ui = Ui {
            ui_tx,
            ui_rx,
            cursive,
            controller_tx,
        };

        ui.cursive.set_autorefresh(true);

        let controller_tx_clone = ui.controller_tx.clone();
        controller_tx_clone
            .send(ControllerMessage::LoadFeeds)
            .unwrap();

        let mut feeds_select: SelectView<u32> = SelectView::new().autojump();
        let mut episodes_select: SelectView<u32> = SelectView::new();

        // Configure callback for feeds list
        let controller_tx_clone = ui.controller_tx.clone();
        feeds_select.set_on_submit(move |s, p: &u32| {
            controller_tx_clone
                .send(ControllerMessage::UpdateSelectedFeed(*p))
                .unwrap();
            s.focus_name("episodes").unwrap();
        });

        let controller_tx_clone = ui.controller_tx.clone();
        episodes_select.set_on_submit(move |_, id: &u32| {
            controller_tx_clone
                .send(ControllerMessage::UpdatePlayEpisode(*id))
                .unwrap();
        });

        let controller_tx_clone = ui.controller_tx.clone();
        ui.cursive.add_global_callback('p', move |_| {
            controller_tx_clone
                .send(ControllerMessage::UpdatePlayState)
                .unwrap();
        });

        let controller_tx_clone = ui.controller_tx.clone();
        ui.cursive.add_global_callback('s', move |_| {
            controller_tx_clone
                .send(ControllerMessage::UpdateStopPlayer)
                .unwrap();
        });

        let controller_tx_clone = ui.controller_tx.clone();
        ui.cursive.add_global_callback('r', move |s| {
            let feed = s.find_name::<SelectView<u32>>("feeds").unwrap();
            match feed.selection() {
                Some(id) => {
                    controller_tx_clone
                        .send(ControllerMessage::ReloadFeedEpisodes(*id))
                        .unwrap();
                }
                _ => (),
            };
        });

        let controller_tx_clone = ui.controller_tx.clone();
        ui.cursive.add_global_callback('d', move |s| {
            let feed = s.find_name::<SelectView<u32>>("feeds").unwrap();
            match feed.selection() {
                Some(id) => {
                    controller_tx_clone
                        .send(ControllerMessage::DeleteFeed(*id))
                        .unwrap();
                }
                _ => (),
            };
        });

        let controller_tx_clone = ui.controller_tx.clone();
        ui.cursive.add_fullscreen_layer(
            OnEventView::new(
                LinearLayout::vertical()
                    .child(
                        LinearLayout::horizontal()
                            .child(
                                Panel::new(feeds_select.with_name("feeds").scrollable())
                                    .title("Feeds")
                                    .full_width(),
                            )
                            .child(
                                Panel::new(episodes_select.with_name("episodes").scrollable())
                                    .title("Episodes")
                                    .full_width(),
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
                                .full_height()
                                .full_width(),
                            ),
                    )
                    .child(Panel::new(
                        ProgressBar::new().with_name("progress").full_width(),
                    )),
            )
            .on_event('a', move |s| {
                let controller_tx_clone = controller_tx_clone.clone();
                add_feed_dialog(s, controller_tx_clone);
            }),
        );

        ui.cursive.add_global_callback('q', |s| s.quit());

        ui
    }

    /// Step the UI by calling into Cursive's step function, then
    /// processing any UI messages.
    pub fn step(&mut self) -> bool {
        if !self.cursive.is_running() {
            return false;
        }

        // Process any pending UI messages
        while let Some(message) = self.ui_rx.try_iter().next() {
            match message {
                UiMessage::UpdateFeeds(feeds) => {
                    self.cursive
                        .call_on_name("feeds", |v: &mut SelectView<u32>| {
                            v.clear();
                            v.add_all(feeds);
                        });
                }
                UiMessage::UpdateEpisodes(episodes) => {
                    self.cursive
                        .call_on_name("episodes", |v: &mut SelectView<u32>| {
                            v.clear();
                            v.add_all(episodes);
                        });
                }
                UiMessage::UpdatePlaying(t, d) => {
                    self.cursive
                        .call_on_name("ep_title", |v: &mut TextView| v.set_content(t));

                    self.cursive
                        .call_on_name("ep_description", |v: &mut TextView| v.set_content(d));
                }
                UiMessage::UpdateProgress(value) => {
                    let mut output = self.cursive.find_name::<ProgressBar>("progress").unwrap();
                    output.set_value(value);
                }
            }
        }

        // Step the UI
        self.cursive.step();

        true
    }
}

fn add_feed_dialog(s: &mut cursive::Cursive, c: mpsc::Sender<ControllerMessage>) {
    s.add_layer(
        OnEventView::new(ResizedView::new(
            SizeConstraint::AtLeast(80),
            SizeConstraint::Free,
            Dialog::new()
                .title("Add New Feed URL")
                .content(
                    ListView::new()
                        .child("URL", EditView::new().with_name("new_feed_url"))
                        .child("Name", EditView::new().with_name("new_feed_name")),
                )
                .button("Cancel", |s| {
                    s.pop_layer();
                })
                .button("Save", move |s| {
                    save_new_feed(s, &c);
                }),
        ))
        .on_event(Key::Esc, |s| {
            s.pop_layer();
        }),
    );
}

fn save_new_feed(s: &mut cursive::Cursive, c: &mpsc::Sender<ControllerMessage>) -> Option<()> {
    let name = s
        .call_on_name("new_feed_name", |v: &mut EditView| v.get_content())?
        .to_string();

    let url = s
        .call_on_name("new_feed_url", |v: &mut EditView| v.get_content())?
        .to_string();

    if name.is_empty() || url.is_empty() {
        return None;
    }

    // do some validation on the url...

    c.send(ControllerMessage::AddNewFeed(name, url)).unwrap();
    s.pop_layer();
    Some(())
}
