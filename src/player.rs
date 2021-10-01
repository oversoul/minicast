/*
use rodio::{Decoder, OutputStream, Sink};
use std::io::Cursor;

pub struct Player {
    pub sink: Sink,
    length: u64,
    cursor: Cursor<Vec<u8>>,
}

impl Player {
    pub fn new(stream_handle: &rodio::OutputStreamHandle) -> Self {
        let sink = Sink::try_new(stream_handle).unwrap();
        let cursor = Cursor::new(Vec::new());
        Player {
            sink,
            cursor,
            length: 0,
        }
    }

    pub fn play_url(&mut self) {
        let ep_url = "https://cdn.simplecast.com/audio/fd7dca0e-5e82-4d04-b65f-c0aa44661798/episodes/fd0bd2ba-c553-466c-a060-b144797ce369/audio/4c6c7b70-68b7-41fe-8280-af8a9d476b0a/default_tc.mp3?aid=embed";
        self.launch_mpv(ep_url);
        /*
        let resp = minreq::get(ep_url).send().expect("not working");
        self.length = resp.as_bytes().len() as u64;
        let output = resp.into_bytes();

        self.cursor = std::io::Cursor::new(output); // Adds Read and Seek to the bytes via Cursor
        let cursor = self.cursor.clone();
        let source = rodio::Decoder::new(cursor).unwrap();

        self.sink.append(source);
        std::thread::sleep(std::time::Duration::new(5, 0));
        // self.sink.pause();
        */
    }

    fn launch_mpv(&self, url: &str) -> Result<(), String> {
        if let Err(err) = std::process::Command::new("mpv")
            .args(&["--no-audio-display", "--ytdl=no", url])
            .status()
        {
            let stderr = std::io::stderr();
            let _handle = stderr.lock();
            match err.kind() {
                std::io::ErrorKind::NotFound => {
                    return Err("Couldn't open mpv\nTrying vlc...".into());
                }
                _ => return Err(format!("Error: {}", err))
            };
        }
        Ok(())
    }

    pub fn play_or_pause(&self) {
        if self.sink.is_paused() {
            self.sink.play();
        } else {
            self.sink.pause();
        }
    }

    pub fn stop(&self) {
        self.sink.stop();
    }

    pub fn position(&self) -> u64 {
        self.cursor.position()
    }

    pub fn length(&self) -> u64 {
        self.length
    }
}
*/

use std::sync::mpsc::{self, TryRecvError};
use std::sync::{Arc, Mutex};

use mpv::{MpvHandler, MpvHandlerBuilder, Result};

pub struct MediaWorker {
    handler: MpvHandler,
    is_paused: bool,
}

impl MediaWorker {
    pub fn new() -> Result<Self> {
        let mut handler = MpvHandlerBuilder::new()?.build()?;
        handler.set_option("vid","no").expect("Failed to set option 'vid' to 'no'");
        Ok(Self {
            handler,
            is_paused: false,
        })
    }

    pub fn loadfile(&mut self, url: &str) -> Result<()> {
        self.handler.command(&["loadfile", &url, "append-play"])?;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        self.handler.command(&["stop"])?;
        Ok(())
    }

    fn next(&mut self) -> Result<()> {
        self.handler.command(&["playlist-next"])?;
        Ok(())
    }

    fn prev(&mut self) -> Result<()> {
        self.handler.command(&["playlist-prev"])?;
        Ok(())
    }

    pub fn toggle_play(&mut self) -> Result<()> {
        self.is_paused ^= true;
        self.handler.set_property("pause", self.is_paused)?;
        Ok(())
    }

    fn time_seek(&mut self, f: impl FnOnce(i64) -> i64) -> Result<()> {
        let pos: i64 = self.handler.get_property("time-pos")?;
        self.handler.set_property("time-pos", f(pos))?;
        Ok(())
    }

    fn playlist_pos(&self) -> Result<usize> {
        let pos: i64 = self.handler.get_property("playlist-pos")?;
        Ok(pos as usize)
    }

    pub fn percentage(&self) -> Result<f64> {
        let percent: f64 = self.handler.get_property("percent-pos")?;
        Ok(percent as f64)
    }

    fn poll_events(&mut self) -> Result<bool> {
        while let Some(ev) = self.handler.wait_event(0.1) {
            match ev {
                mpv::Event::Shutdown | mpv::Event::Idle => {
                    return Ok(false);
                }
                _ => println!("mpv: {:?}", ev),
            }
        }
        Ok(true)
    }
}

/*
pub enum Command {
    Enqueue { track: Track, url: String },
    Stop,
    NextTrack,
    PrevTrack,
    FlipPause,
    Seek(i64),
}

pub struct PlayerState {
    playlist: Vec<Track>,
    current_position: usize,
}

impl PlayerState {
    fn new() -> Self {
        Self {
            playlist: vec![],
            current_position: 0,
        }
    }

    pub fn playlist(&self) -> impl Iterator<Item = &'_ Track> {
        self.playlist.iter()
    }

    pub fn current(&self) -> usize {
        self.current_position
    }
}

pub type State = Arc<Mutex<PlayerState>>;

pub struct Player {
    rx: mpsc::Receiver<Command>,
    state: State,
}

impl Player {
    pub fn new() -> (Self, mpsc::Sender<Command>) {
        let (tx, rx) = mpsc::channel();
        let state = Arc::new(Mutex::new(PlayerState::new()));
        (Self { rx, state }, tx)
    }

    pub fn start_worker(self) -> (State, std::thread::JoinHandle<Result<()>>) {
        let state = self.state.clone();

        let handle = std::thread::spawn(move || {
            let mut worker = MediaWorker::new()?;
            loop {
                worker.poll_events()?;
                match self.rx.try_recv() {
                    Ok(Command::Enqueue { track, url }) => {
                        if let Err(err) = worker.loadfile(&url) {
                            log::error!("cannot load {}: {}, url: {}", track.name, err, url);
                        } else {
                            self.state.lock().unwrap().playlist.push(track);
                        }
                    }
                    Ok(Command::Stop) => {
                        if let Err(err) = worker.stop() {
                            log::error!("cannot stop the track: {}", err);
                        } else {
                            let mut state = self.state.lock().unwrap();
                            state.playlist.clear();
                            state.current_position = 0;
                        }
                    }
                    Ok(Command::NextTrack) => {
                        if let Err(err) = worker.next() {
                            log::error!("cannot switch to next track: {}", err);
                        } else {
                            self.state.lock().unwrap().current_position += 1;
                        }
                    }
                    Ok(Command::PrevTrack) => {
                        if let Err(err) = worker.prev() {
                            log::error!("cannot switch to previous track: {}", err);
                        } else {
                            self.state.lock().unwrap().current_position -= 1;
                        }
                    }
                    Ok(Command::FlipPause) => {
                        if let Err(err) = worker.flip_pause() {
                            log::error!("cannot pause/unpause track: {}", err);
                        }
                    }
                    Ok(Command::Seek(x)) => {
                        if let Err(err) = worker.time_seek(|pos| pos + x) {
                            log::error!("cannot seek time ({} secs): {}", x, err);
                        }
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => {
                        log::warn!("player command stream disconnected, finishing");
                        return Ok(());
                    }
                }

                if let Ok(pos) = worker.playlist_pos() {
                    let mut state = self.state.lock().unwrap();
                    state.current_position = pos;
                } else {
                    let mut state = self.state.lock().unwrap();
                    state.playlist.clear();
                    state.current_position = 0;
                }
            }
        });

        (state, handle)
    }
}
*/
