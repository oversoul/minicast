#![allow(dead_code)]
extern crate minreq;
extern crate roxmltree;

use std::io::prelude::*;
use std::path::PathBuf;

pub enum Feed {
    Url(String),
    Path(PathBuf),
}

#[derive(Debug, PartialEq, Clone)]
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
/*

impl Feed {
    pub fn from_url<T: Into<String>>(name: T, url: T) -> Self {
        Feed {
            url: url.into(),
            name: name.into(),
            is_url: true,
            path: PathBuf::new(),
            episodes: vec![],
            channel: Channel::new(),
        }
    }

    pub fn from_path<T: Into<String>>(name: T, path: PathBuf) -> Self {
        Feed {
            path,
            name: name.into(),
            url: String::new(),
            is_url: false,
            channel: Channel::new(),
            episodes: vec![],
        }
    }

    pub fn get_episode_by_index(&self, idx: usize) -> Episode {
        Episode::new_from_ref(&self.episodes[idx])
    }

    pub fn get_episodes_title(&self) -> Vec<String> {
        self.episodes
            .iter()
            .map(|e| format!("{}", e.title))
            .collect()
    }

    pub fn parse_episodes(&mut self) -> Vec<Episode> {
        if self.is_url {
            self.parse_url_episodes()
        } else {
            self.parse_path_episodes()
        }
    }

    fn parse_path_episodes(&mut self) -> Vec<Episode> {
        let path = self.path.to_path_buf();
        let file = std::fs::File::open(path).expect("File not found");

        let mut reader = std::io::BufReader::new(file);
        let mut xml = String::new();
        reader.read_to_string(&mut xml).expect("couldn't read file");

        self.parse_xml_string(&xml)
    }

    fn parse_xml_string(&mut self, xml: &str) -> Vec<Episode> {
        let doc: Result<roxmltree::Document, roxmltree::Error> = roxmltree::Document::parse(&xml);

        if doc.is_err() {
            return vec![];
        }

        let doc = doc.unwrap();
        let rss: roxmltree::Node = doc.root().first_child().unwrap();

        // validate the root tag...
        if rss.tag_name().name() != "rss" {
            return vec![];
        }
        // validate the rss version
        if rss.attribute("version") != Some("2.0") {
            return vec![];
        }

        let mut episodes = vec![];

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
                            episodes.push(episode);
                        }
                    }
                    _ => (),
                }
            }
        }

        episodes
        // panic if there is no channel
    }

    fn parse_url_episodes(&mut self) -> Vec<Episode> {
        let response: Result<minreq::Response, minreq::Error> = minreq::get(&self.url).send();
        if response.is_err() {
            return vec![];
        }

        let response = response.unwrap();

        if response.status_code != 200 {
            return vec![];
        }

        match response.as_str() {
            Ok(content) => self.parse_xml_string(content),
            Err(_) => vec![],
        }
    }
}
*/

fn get_element_text<'a>(element: &'a roxmltree::Node) -> &'a str {
    match element.first_child() {
        Some(child) => child.text().unwrap_or(""),
        None => "",
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
            "enclosure" => url = item_child.attribute("url")?.into(),
            "description" => description = item_child.first_child()?.text()?.into(),
            _ => (),
        }
    }

    if url == "" {
        return None;
    }

    Some(Episode::new(title, description, url))
}

pub fn get_episodes(feed: Feed) -> Vec<Episode> {
    match feed {
        Feed::Path(path) => parse_path_episodes(path),
        Feed::Url(url) => parse_url_episodes(url),
    }
}

fn parse_xml_string(xml: &str) -> Vec<Episode> {
    let doc: Result<roxmltree::Document, roxmltree::Error> = roxmltree::Document::parse(&xml);

    if doc.is_err() {
        return vec![];
    }

    let doc = doc.unwrap();
    let rss: roxmltree::Node = doc.root().first_child().unwrap();

    // validate the root tag...
    if rss.tag_name().name() != "rss" {
        return vec![];
    }
    // validate the rss version
    if rss.attribute("version") != Some("2.0") {
        return vec![];
    }

    let mut episodes = vec![];

    for child in rss.children() {
        if child.node_type() != roxmltree::NodeType::Element {
            continue;
        }

        if child.tag_name().name() != "channel" {
            continue;
        }

        for sub_child in child.children() {
            match sub_child.tag_name().name() {
                // "link" => self.channel.link = get_element_text(&sub_child).into(),
                // "title" => self.channel.title = get_element_text(&sub_child).into(),
                // "description" => self.channel.description = get_element_text(&sub_child).into(),
                "item" => {
                    if let Some(episode) = item_to_episode(&sub_child) {
                        episodes.push(episode);
                    }
                }
                _ => (),
            }
        }
    }

    episodes
    // panic if there is no channel
}

fn parse_url_episodes(url: String) -> Vec<Episode> {
    let response: Result<minreq::Response, minreq::Error> = minreq::get(&url).send();
    if response.is_err() {
        return vec![];
    }

    let response = response.unwrap();

    if response.status_code != 200 {
        return vec![];
    }

    match response.as_str() {
        Ok(content) => parse_xml_string(content),
        Err(_) => vec![],
    }
}

fn parse_path_episodes(path: PathBuf) -> Vec<Episode> {
    let file = std::fs::File::open(path).expect("File not found");

    let mut reader = std::io::BufReader::new(file);
    let mut xml = String::new();
    reader.read_to_string(&mut xml).expect("couldn't read file");

    parse_xml_string(&xml)
}

#[cfg(test)]
use std::env;
#[cfg(test)]
use std::path::Path;

#[test]
fn can_parse_xml_files() {
    let root_path = env::current_dir().expect("something is wrong with finding current dir.");
    let path = root_path.join("feeds/valid_basic.xml");
    let feed = Feed::Path(path.to_path_buf());
    let episodes = get_episodes(feed);
    assert_eq!(episodes.len(), 3);
}

#[test]
fn test_feed_validation_complete() {
    let root_path = env::current_dir().expect("something is wrong with finding current dir.");
    let path = root_path.join("feeds/valid_basic.xml");
    let feed = Feed::Path(path.to_path_buf());
    let episodes = get_episodes(feed);
    assert_eq!(episodes.len(), 3);
}

#[test]
fn test_feed_validation_valid_mixed_enclosure() {
    let root_path = env::current_dir().expect("something is wrong with finding current dir.");
    let path = root_path.join("feeds/valid_mixed_enclosures.xml");
    let feed = Feed::Path(path.to_path_buf());
    let episodes = get_episodes(feed);
    assert_eq!(episodes.len(), 2);
}

#[test]
fn test_feed_validations_is_rss() {
    let root_path = env::current_dir().expect("something is wrong with finding current dir.");
    let path = root_path.join("feeds/broken_is_rss.xml");
    let feed = Feed::Path(path.to_path_buf());
    let episodes = get_episodes(feed);
    assert_eq!(episodes.len(), 0);
}

#[test]
fn test_feed_validations_is_v2() {
    let root_path = env::current_dir().expect("something is wrong with finding current dir.");
    let path = root_path.join("feeds/broken_is_v2.xml");

    let feed = Feed::Path(path.to_path_buf());
    let episodes = get_episodes(feed);
    assert_eq!(episodes.len(), 0);
}

#[test]
fn test_feed_validations_rss_empty() {
    let root_path = env::current_dir().expect("something is wrong with finding current dir.");
    let path = root_path.join("feeds/broken_rss_empty.xml");

    let feed = Feed::Path(path.to_path_buf());
    let episodes = get_episodes(feed);
    assert_eq!(episodes.len(), 0);
}

#[test]
fn test_feed_validations_has_channel() {
    let root_path = env::current_dir().expect("something is wrong with finding current dir.");
    let path = root_path.join("feeds/broken_has_channel.xml");

    let feed = Feed::Path(path.to_path_buf());
    let episodes = get_episodes(feed);
    assert_eq!(episodes.len(), 0);
}

#[test]
fn test_feed_validations_channel_children() {
    let root_path = env::current_dir().expect("something is wrong with finding current dir.");
    let path = root_path.join("feeds/broken_channel_children.xml");

    let feed = Feed::Path(path.to_path_buf());
    let episodes = get_episodes(feed);
    assert_eq!(episodes.len(), 0);
}

#[test]
fn test_feed_validations_channel_empty() {
    let root_path = env::current_dir().expect("something is wrong with finding current dir.");
    let path = root_path.join("feeds/broken_channel_empty.xml");

    let feed = Feed::Path(path.to_path_buf());
    let episodes = get_episodes(feed);
    assert_eq!(episodes.len(), 0);
}

#[test]
fn test_feed_validations_two_channels() {
    let root_path = env::current_dir().expect("something is wrong with finding current dir.");
    let path = root_path.join("feeds/valid_two_channels.xml");

    let feed = Feed::Path(path.to_path_buf());
    let episodes = get_episodes(feed);
    assert_eq!(episodes.len(), 3);
}

#[test]
fn test_feed_validations_item_title() {
    let root_path = env::current_dir().expect("something is wrong with finding current dir.");
    let path = root_path.join("feeds/broken_item_title.xml");

    let feed = Feed::Path(path.to_path_buf());
    let episodes = get_episodes(feed);
    assert_eq!(episodes.len(), 0);
}

#[test]
#[should_panic]
fn test_feed_load_error() {
    let path = Path::new("notfound");
    let feed = Feed::Path(path.to_path_buf());
    let _episodes = get_episodes(feed);
}
