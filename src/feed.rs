#![allow(dead_code)]
extern crate minreq;
extern crate roxmltree;

use std::io::prelude::*;
#[warn(unused_imports)]
use std::path::Path;
use std::path::PathBuf;

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

#[derive(Debug, PartialEq)]
pub struct Episode {
    pub url: String,
    pub title: String,
    pub description: String,
}

impl Episode {
    fn new<T: Into<String>>(title: T, description: T, url: T) -> Self {
        Episode {
            title: title.into(),
            description: description.into(),
            url: url.into(),
        }
    }
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

        self.parse_xml_string(&xml);
    }

    fn parse_xml_string(&mut self, xml: &str) {
        let doc: roxmltree::Document = roxmltree::Document::parse(&xml).unwrap();
        let rss: roxmltree::Node = doc.root().first_child().unwrap();

        // validate the root tag...
        if rss.tag_name().name() != "rss" {
            return;
        }
        // validate the rss version
        if rss.attribute("version") != Some("2.0") {
            return;
        }

        for child in rss.children() {
            if child.node_type() != roxmltree::NodeType::Element {
                continue;
            }

            if child.tag_name().name() != "channel" {
                continue;
            }

            for sub_child in child.children() {
                match sub_child.tag_name().name() {
                    "link" => self.channel.link = get_element_text(&sub_child).into(),
                    "title" => self.channel.title = get_element_text(&sub_child).into(),
                    "description" => self.channel.description = get_element_text(&sub_child).into(),
                    "item" => {
                        if let Some(episode) = item_to_episode(&sub_child) {
                            self.episodes.push(episode);
                        }
                    }
                    _ => (),
                }
            }
        }

        // panic if there is no channel
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

        self.parse_xml_string(content);
    }
}

fn get_element_text<'a>(element: &'a roxmltree::Node) -> &'a str {
    match element.first_child() {
        Some(child) => child.text().unwrap_or(""),
        None => ""
    }
}

fn item_to_episode(element: &roxmltree::Node) -> Option<Episode> {
    let item = element.children();

    let mut url = String::new();
    let mut title = String::new();
    let mut description = String::new();

    for item_child in item {
        if item_child.tag_name().name() == "" {
            continue;
        }

        match item_child.tag_name().name() {
            "title" => title = item_child.first_child()?.text()?.into(),
            "description" => description = item_child.first_child()?.text()?.into(),
            "enclosure" => url = item_child.attribute("url")?.into(),
            _ => (),
        }
    }

    if url == "" {
        return None;
    }

    Some(Episode::new(title, description, url))
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

#[test]
fn test_feed_validations_two_channels() {
    let path = Path::new("/mnt/ddrive/rust/cast/feeds/valid_two_channels.xml");
    let mut feed = Feed::from_path(path.to_path_buf());
    feed.parse_episodes();
    assert_eq!(feed.episodes.len(), 3);
}

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
