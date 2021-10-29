use rusqlite::{params, Connection, Result};

pub struct Database {
    connection: Connection,
}

#[derive(Debug)]
pub struct Feed {
    pub id: u32,
    pub url: String,
    pub name: String,
}

#[derive(Debug)]
pub struct Episode {
    pub id: u32,
    pub url: String,
    pub title: String,
    pub description: String,
}

impl Database {
    pub fn new() -> Result<Self> {
        // let conn = Connection::open_in_memory()?;
        let conn = Connection::open("./db.sqlite")?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS feeds (
               id              INTEGER PRIMARY KEY,
               url             TEXT NOT NULL,
               name            TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS episodes (
               id              INTEGER PRIMARY KEY,
               url             TEXT NOT NULL,
               title           TEXT NOT NULL,
               description     TEXT NOT NULL,
               feed_id         INTEGER
            )",
            [],
        )?;

        let feeds = [
            ("Laracasts", "https://feeds.simplecast.com/sY509q85"),
            ("TED Talks Daily", "https://www.ted.com/feeds/talks.rss"),
            ("Smartless", "https://rss.art19.com/smartless"),
            ("Invisibllia", "https://feeds.npr.org/510307/podcast.xml"),
        ];

        /*
        for (name, url) in feeds {
            conn.execute(
                "INSERT INTO feeds (url, name) VALUES (?1, ?2)",
                params![url, name],
            )?;
        }
        */

        Ok(Self { connection: conn })
    }

    pub fn get_episode(&self, id: u32) -> std::result::Result<Episode, String> {
        let mut stmt = self
            .connection
            .prepare("SELECT id, url, title, description FROM episodes WHERE id = ?1")
            .expect("couldn't run query");

        let row = stmt.query_row(params![id], |row| {
            Ok(Episode {
                id: row.get(0).unwrap_or(0),
                url: row.get(1).unwrap_or(String::from("")),
                title: row.get(2).unwrap_or(String::from("")),
                description: row.get(3).unwrap_or(String::from("")),
            })
        });

        match row {
            Ok(episode) => Ok(episode),
            Err(_) => Err("no episode.".into()),
        }
    }

    pub fn create_feed(&self, name: String, url: String) -> Result<u32> {
        self.connection.execute(
            "INSERT INTO feeds (url, name) VALUES (?1, ?2)",
            params![url, name],
        )?;
        Ok(self.connection.last_insert_rowid() as u32)
    }

    pub fn get_feed(&self, id: u32) -> std::result::Result<Feed, String> {
        let mut stmt = self
            .connection
            .prepare("SELECT id, url, name FROM feeds WHERE id = ?1")
            .expect("couldn't run query");

        let row = stmt.query_row(params![id], |row| {
            Ok(Feed {
                id: row.get(0).unwrap_or(0),
                url: row.get(1).unwrap_or(String::from("")),
                name: row.get(2).unwrap_or(String::from("")),
            })
        });

        match row {
            Ok(feed) => Ok(feed),
            Err(_) => Err("no feed".into()),
        }
    }

    pub fn get_feeds(&self) -> Vec<Feed> {
        let mut stmt = self
            .connection
            .prepare("SELECT id, url, name FROM feeds")
            .expect("couldn't run query");

        let rows = stmt.query_map([], |row| {
            Ok(Feed {
                id: row.get(0).unwrap_or(0),
                url: row.get(1).unwrap_or(String::from("")),
                name: row.get(2).unwrap_or(String::from("")),
            })
        });

        match rows {
            Ok(map) => map
                .filter_map(|it| match it {
                    Ok(row) => Some(row),
                    _ => None,
                })
                .collect(),
            _ => Vec::new(),
        }
    }

    pub fn get_episodes(&self, feed_id: u32) -> Vec<Episode> {
        let mut stmt = self
            .connection
            .prepare("SELECT id, url, title, description, feed_id FROM episodes WHERE feed_id = ?1")
            .expect("couldn't run query");

        let rows = stmt.query_map(params![feed_id], |row| {
            Ok(Episode {
                id: row.get(0).unwrap_or(0),
                url: row.get(1).unwrap_or(String::from("")),
                title: row.get(2).unwrap_or(String::from("")),
                description: row.get(3).unwrap_or(String::from("")),
            })
        });

        match rows {
            Ok(map) => map
                .filter_map(|it| match it {
                    Ok(row) => Some(row),
                    _ => None,
                })
                .collect(),
            _ => Vec::new(),
        }
    }

    pub fn delete_feed(&self, feed_id: u32) -> Result<()> {
        self.connection
            .execute("DELETE from feeds WHERE id = ?1", params![feed_id])?;
        self.clear_episodes(feed_id)?;
        Ok(())
    }

    pub fn clear_episodes(&self, feed_id: u32) -> Result<()> {
        self.connection
            .execute("DELETE from episodes WHERE feed_id = ?1", params![feed_id])?;
        Ok(())
    }

    pub fn set_episodes(&self, feed_id: u32, episodes: Vec<Episode>) -> Result<()> {
        for episode in episodes {
            self.connection.execute(
                "INSERT INTO episodes (url, title, description, feed_id) VALUES (?1, ?2, ?3, ?4)",
                params![episode.url, episode.title, episode.description, feed_id],
            )?;
        }
        Ok(())
    }
}
