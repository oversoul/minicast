#![allow(dead_code)]
#![warn(unused_imports)]
extern crate cursive;

mod app;
mod controller;
mod db;
mod feed;
mod player;
mod ui;

fn main() {
    let controller = controller::Controller::new();
    match controller {
        Ok(mut controller) => controller.run(),
        Err(e) => println!("Error: {}", e),
    };
}
