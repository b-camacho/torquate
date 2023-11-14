// Alters the real world state of the lights
// this file is the boundary between bevy ECS code and regular code
use bevy::prelude::*;
use reqwest::blocking::Client;
use reqwest::Error;
use std::sync::{Arc, Mutex};
use std::{thread, time::Duration};

struct State {
    brightness: u8
}

#[derive(Resource)]
pub struct BulbState {
    inner: Arc<Mutex<State>>,
}
impl BulbState {
    pub fn set_brightness(&self, brightness: u8) {
        self.inner.lock().unwrap().brightness = brightness;
    }
}

fn update_brightness(state: Arc<Mutex<State>>) -> Result<(), Error> {
    let url_base = std::env::var("HUE_URL").expect("set HUE_URL env var to the bridge API endpoint");
    let url_state = format!("{url_base}/lights/1/state");
    println!("url: {url_state}");
    let mut last_sent: Option<u8> = None;
    let client = Client::new();
    loop {
        let brightness = { state.lock().unwrap().brightness };
        if last_sent.is_none() || last_sent.unwrap() != brightness {
            println!("egress: {brightness}");
            let body = format!(r#"{{"bri": {}}}"#, brightness);
            //client.put(&url_state)
            //    .body(body)
            //    .send()?;
            last_sent = Some(brightness);
        }
        thread::sleep(Duration::from_millis(10));
    }
}

pub struct HuePlugin;

impl Plugin for HuePlugin {
    fn build(&self, app: &mut App) {
        let state = Arc::new(Mutex::new(State{brightness: 0}));
        app.world.insert_resource(BulbState{inner: state.clone() });

        thread::spawn(move || {
            update_brightness(state).unwrap_or_else(|e| eprintln!("Background task failed: {}", e));
        });
    }
}

