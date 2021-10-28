use libmpv::{Mpv, Result};

pub struct MediaWorker {
    handler: Mpv,
    pub is_paused: bool,
}

pub fn seconds_to_hms(position: i64) -> String {
    let duration = std::time::Duration::new(position as u64, 0);
    let seconds = duration.as_secs() % 60;
    let minutes = (duration.as_secs() / 60) % 60;
    let hours = (duration.as_secs() / 60) / 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

impl MediaWorker {
    pub fn new() -> Result<Self> {
        let handler = Mpv::new().expect("coudln't start mpv");
        handler
            .set_property("vid", "no")
            .expect("Failed to set option 'vid' to 'no'");

        Ok(Self {
            handler,
            is_paused: false,
        })
    }

    pub fn loadfile(&mut self, url: &str) -> Result<()> {
        self.handler.command("loadfile", &[url])?;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        self.handler.command("stop", &[])?;
        Ok(())
    }

    pub fn quit(&mut self) -> Result<()> {
        self.handler.command("quit", &[])?;
        Ok(())
    }

    fn next(&mut self) -> Result<()> {
        self.handler.command("playlist-next", &[])?;
        Ok(())
    }

    fn prev(&mut self) -> Result<()> {
        self.handler.command("playlist-prev", &[])?;
        Ok(())
    }

    pub fn toggle_play(&mut self) -> Result<()> {
        if self.is_paused {
            self.handler.unpause()?;
        } else {
            self.handler.pause()?;
        }
        self.is_paused = !self.is_paused;
        Ok(())
    }

    pub fn time_position(&self) -> Result<String> {
        let pos: i64 = self.handler.get_property("time-pos")?;
        let duration: i64 = self.handler.get_property("duration")?;

        Ok(format!(
            "[{}/{}]",
            seconds_to_hms(pos),
            seconds_to_hms(duration)
        ))
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

    pub fn percent(&self) -> Result<usize> {
        let percent: f64 = self.handler.get_property("percent-pos")?;
        Ok(percent as usize)
    }

    pub fn percentage(&self) -> Result<f64> {
        let percent: f64 = self.handler.get_property("percent-pos")?;
        Ok(percent as f64)
    }
}
