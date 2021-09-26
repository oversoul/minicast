#![allow(dead_code)]
extern crate minreq;
extern crate quick_xml;
extern crate serde;

use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;

use quick_xml::de::{from_str, DeError};
use serde::{Deserialize, Deserializer};

pub struct Feed {
    is_url: bool,
    url: String,
    path: PathBuf,
    channel: Channel,
    pub episodes: Vec<Episode>,
}

#[derive(Debug, PartialEq)]
pub struct Channel {
    title: String,
    description: String,
    link: String,
}

impl Channel {
    fn new() -> Self {
        Channel {
            title: "".into(),
            description: "".into(),
            link: "".into(),
        }
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Enclosure {
    pub url: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Episode {
    #[serde(rename="guid", default)]
    guid: String,
    // title: String,
    #[serde(default)]
    pub description: String,
    pub enclosure: Option<Enclosure>,
}


#[derive(Debug, Deserialize)]
struct Link {
    #[serde(rename = "$value")]
    value: String,
}

#[derive(Debug, Deserialize, PartialEq)]
struct ChannelXml {
    title: String,
    description: String,

    #[serde(rename="$value")]
    link: std::collections::BTreeMap<String, String>,

    #[serde(rename = "item")]
    episodes: Vec<Episode>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct PodcastXml {
    channel: ChannelXml,
}

impl Feed {
    pub fn from_url<T: Into<String>>(url: T) -> Self {
        Feed {
            url: url.into(),
            path: PathBuf::new(),
            is_url: true,
            channel: Channel::new(),
            episodes: vec![],
        }
        // validate?
    }

    pub fn from_path(path: PathBuf) -> Self {
        Feed {
            path,
            url: String::new(),
            is_url: false,
            channel: Channel::new(),
            episodes: vec![],
        }
        // validate?
    }

    pub fn parse_episodes(&mut self) {
        if self.is_url {
            self.parse_url_episodes();
        } else {
            self.parse_path_episodes();
        }
    }

    fn parse_path_episodes(&mut self) {
        let path = self.path.to_path_buf();
        let file = std::fs::File::open(path).expect("Couldn't open file");
        let mut reader = std::io::BufReader::new(file);
        let mut xml = String::new();
        reader.read_to_string(&mut xml).expect("couldn't read file");

        let podcast: Option<PodcastXml> = match from_str(&xml) {
            Ok(value) => value,
            Err(e) => {
                println!("{:?}", e);
                None
            }
        };

        if podcast.is_none() {
            return;
        }

        let podcast = podcast.unwrap();

        let episodes: Vec<Episode> = podcast
            .channel
            .episodes
            .into_iter()
            .filter(|e| e.enclosure.is_some())
            .collect();

        self.channel = Channel {
            link: String::new(), //podcast.channel.link,
            title: podcast.channel.title,
            description: podcast.channel.description,
        };

        self.episodes = episodes;
    }

    fn parse_url_episodes(&mut self) {
        let response: minreq::Response = minreq::get(&self.url)
            .send()
            .expect("something went wrong url");
        if response.status_code != 200 {
            println!("response status is not 200");
            return;
        }

        let content = response
            .as_str()
            .expect("couldn't parse response to string");
        //println!("{}", content);

        let podcast: Option<PodcastXml> = match from_str(&content) {
            Ok(value) => value,
            Err(e) => {
                println!("{:?}", e);
                None
            }
        };

        if podcast.is_none() {
            return;
        }

        let podcast = podcast.unwrap();

        let episodes: Vec<Episode> = podcast
            .channel
            .episodes
            .into_iter()
            .filter(|e| e.enclosure.is_some())
            .collect();

        self.channel = Channel {
            link: String::new(), //podcast.channel.link,
            title: podcast.channel.title,
            description: podcast.channel.description,
        };

        self.episodes = episodes;
    }
}

#[test]
fn can_create_new_instance_from_url() {
    let url = "http://example.com";
    let feed = Feed::from_url(url);
    assert_eq!(feed.is_url, true);
    assert_eq!(feed.url, url);
}

#[test]
fn can_create_new_instance_from_path() {
    let path = Path::new("../feeds/valid_basic.xml");
    let feed = Feed::from_path(path.to_path_buf());
    assert_eq!(feed.is_url, false);
    assert_eq!(feed.path, path);
}

#[test]
fn can_parse_xml_files() {
    let path = Path::new("/mnt/ddrive/rust/cast/feeds/valid_basic.xml");
    let mut feed = Feed::from_path(path.to_path_buf());
    feed.parse_episodes();
    assert_eq!(feed.episodes.len(), 3);
}

#[test]
fn test_feed_validation_complete() {
    let path = Path::new("/mnt/ddrive/rust/cast/feeds/valid_complete.xml");
    let mut feed = Feed::from_path(path.to_path_buf());
    feed.parse_episodes();
    assert_eq!(feed.episodes.len(), 3);
}

#[test]
fn test_feed_validation_valid_mixed_enclosure() {
    let path = Path::new("/mnt/ddrive/rust/cast/feeds/valid_mixed_enclosures.xml");
    let mut feed = Feed::from_path(path.to_path_buf());
    feed.parse_episodes();
    assert_eq!(feed.episodes.len(), 2);
}

#[test]
fn test_feed_validations_is_rss() {
    let path = Path::new("/mnt/ddrive/rust/cast/feeds/broken_is_rss.xml");
    let mut feed = Feed::from_path(path.to_path_buf());
    feed.parse_episodes();
    assert_eq!(feed.episodes.len(), 0);
}

#[test]
fn test_feed_validations_is_v2() {
    let path = Path::new("/mnt/ddrive/rust/cast/feeds/broken_is_v2.xml");
    let mut feed = Feed::from_path(path.to_path_buf());
    feed.parse_episodes();
    assert_eq!(feed.episodes.len(), 0);
}

#[test]
fn test_feed_validations_rss_empty() {
    let path = Path::new("/mnt/ddrive/rust/cast/feeds/broken_rss_empty.xml");
    let mut feed = Feed::from_path(path.to_path_buf());
    feed.parse_episodes();
    // should show an error
    assert_eq!(feed.episodes.len(), 0);
}

#[test]
fn test_feed_validations_has_channel() {
    let path = Path::new("/mnt/ddrive/rust/cast/feeds/broken_has_channel.xml");
    let mut feed = Feed::from_path(path.to_path_buf());
    feed.parse_episodes();
    assert_eq!(feed.episodes.len(), 0);
}

#[test]
fn test_feed_validations_channel_children() {
    let path = Path::new("/mnt/ddrive/rust/cast/feeds/broken_channel_children.xml");
    let mut feed = Feed::from_path(path.to_path_buf());
    feed.parse_episodes();
    assert_eq!(feed.episodes.len(), 0);
}

#[test]
fn test_feed_validations_channel_empty() {
    let path = Path::new("/mnt/ddrive/rust/cast/feeds/broken_channel_empty.xml");
    let mut feed = Feed::from_path(path.to_path_buf());
    feed.parse_episodes();
    assert_eq!(feed.episodes.len(), 0);
}

/* INVALID for now...
#[test]
fn test_feed_validations_two_channels() {
    let path = Path::new("/mnt/ddrive/rust/cast/feeds/valid_two_channels.xml");
    let mut feed = Feed::from_path(path.to_path_buf());
    feed.parse_episodes();
    println!("{:#?}", feed.episodes);
}
*/

#[test]
fn test_feed_validations_item_title() {
    let path = Path::new("/mnt/ddrive/rust/cast/feeds/broken_item_title.xml");
    let mut feed = Feed::from_path(path.to_path_buf());
    feed.parse_episodes();
    assert_eq!(feed.episodes.len(), 0);
}

#[test]
#[should_panic]
fn test_feed_load_error() {
    let path = Path::new("notfound");
    let mut feed = Feed::from_path(path.to_path_buf());
    feed.parse_episodes();
}
