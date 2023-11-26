// Alters the real world state of the lights
// this file is the boundary between bevy ECS code and regular code
use anyhow::Error;
use bevy::prelude::*;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::{thread, time::Duration};
use crate::util::MapRange;

struct Conn {
    url_base: String,
    mode: ConnMode,
    client: Client,
}
enum ConnMode {
    LogOnly,   // log what you would send instead of sending the request
    ReadOnly,  // send read requests, but log writes
    ReadWrite, // "normal" mode
}

#[derive(Serialize, Deserialize)]
struct JsonState {
    on: bool,
    bri: u8,
    hue: Option<u16>,
    // Other fields are omitted for brevity
}

#[derive(Serialize, Deserialize)]
struct JsonBulb {
    state: JsonState,
    uniqueid: String,
    // Other fields are omitted for brevity
}

fn parse_state(json_str: &str) -> Result<Vec<BulbRead>, Error> {
    let bulbs: HashMap<String, JsonBulb> = serde_json::from_str(json_str)?;
    let mut bulb_reads = Vec::new();

    for (idx, bulb) in bulbs.iter() {
        if let Some(hue) = bulb.state.hue {
            bulb_reads.push(BulbRead {
                brightness: bulb.state.bri as f64,
                hue: (hue as f64).map((0f64, u16::MAX as f64), (0f64, 1f64)),
                idx: idx.parse::<u8>().unwrap_or_default(),
                uuid: bulb.uniqueid.clone(),
                on: bulb.state.on,
            });
        }
    }

    Ok(bulb_reads)
}

impl Conn {
    pub fn new(mode: ConnMode) -> Self {
        let url_base =
            std::env::var("HUE_URL").expect("set HUE_URL env var to the bridge API endpoint");

        let client = Client::new();
        Self {
            mode,
            url_base,
            client,
        }
    }

    pub fn set_bulb_state(&self, idx: u8, brightness: f64, hue: f64) -> Result<(), Error> {
        let url_base = &self.url_base; // can't have . in {} yet
        let url_state = format!("{url_base}/lights/{idx}/state");
        let brightness = brightness.map((0f64, 1f64), (0f64, 255f64)) as u8;
        let hue = hue.map((0f64, 1f64), (0f64, u16::MAX as f64)) as u16;
        let body = format!(r#"{{"bri": {brightness}, "hue": {hue}}}"#);
        match self.mode {
            ConnMode::ReadWrite => {
                self.client.put(url_state).body(body).send()?;
            }
            _ => {
                println!("egress: {}", body)
            }
        }
        Ok(())
    }

    pub fn get_state(&self) -> Result<Vec<BulbRead>, Error> {
        let url_base = &self.url_base;
        let url_lights = format!("{url_base}/lights");

        match self.mode {
            ConnMode::ReadOnly | ConnMode::ReadWrite => {
                let lights_json = self.client.get(url_lights).send()?.text()?;
                parse_state(&lights_json)
            }
            ConnMode::LogOnly => {
                let file = std::fs::read("../example_bridge_response.json").unwrap();
                let lights_json = std::str::from_utf8(&file).unwrap();
                parse_state(lights_json)
            }
        }
    }
}

// Why keep most of the data twice?
// `writes` represents desired bulb state
// `reads` represents values we read back from the hue system
// these can differ because lights will take time to change hue and brightness
// `BulbRead` also includes attributes we can't change, like spatial position and index
#[derive(Clone, Default)]
struct State {
    ready: bool,
    writes: Vec<BulbWrite>,
    reads: Vec<BulbRead>,
    brightness: u8,
}

#[derive(Clone, Default, PartialEq)]
pub struct BulbWrite {
    pub idx: u8,
    pub brightness: f64, // 0 - fully off, 1 - fully on
    pub hue: f64,        // hue as in HSV
}

#[derive(Clone, Default)]
pub struct BulbRead {
    pub brightness: f64,
    pub hue: f64,
    pub idx: u8,
    pub uuid: String,
    pub on: bool,
    //TODO: position in space
}

#[derive(Resource)]
pub struct BulbState {
    inner: Arc<Mutex<State>>,
}
impl BulbState {
    pub fn set_brightness(&self, idx: u8, brightness: f64) {
        let mut state = self.inner.lock().unwrap();
        if let Some(bulb) = state.writes.iter_mut().find(|w| w.idx == idx) {
            bulb.brightness = brightness
        }
    }

    pub fn ready(&self) -> bool {
        self.inner.lock().unwrap().ready
    }

    pub fn reads(&self) -> Vec<BulbRead> {
        self.inner.lock().unwrap().reads.clone()
    }
}

fn has_delta(r: &BulbRead, w: &BulbWrite) -> bool {
    (r.hue - w.hue).abs() > f64::EPSILON || (r.brightness - w.brightness).abs() > f64::EPSILON
}

fn run(state: Arc<Mutex<State>>, conn: Conn) -> Result<(), Error> {
    let mut last_sent: Option<Vec<(u8, BulbWrite)>> = None;
    let bulbs = conn.get_state()?;
    {
        let mut state = state.lock().unwrap();
        state.writes = bulbs
            .iter()
            .map(|br| BulbWrite {
                idx: br.idx,
                brightness: br.brightness,
                hue: br.hue,
            })
            .collect();
        state.reads = bulbs;
        state.ready = true;
    }

    loop {
        let updates: Vec<(u8, BulbWrite)> = {
            let state = state.lock().unwrap();
            state
                .reads
                .iter()
                .zip(state.writes.iter())
                .filter(|(r, w)| has_delta(r, w))
                .map(|(r, w)| (r.idx, w.clone()))
                .collect()
        };

        if last_sent.is_none() || last_sent.as_ref().unwrap() != &updates {
            for (idx, update) in updates.clone() {
                conn.set_bulb_state(idx, update.hue, update.brightness)?
            }
            last_sent = Some(updates);
        }

        thread::sleep(Duration::from_millis(10));
    }
}

pub struct HuePlugin;

impl Plugin for HuePlugin {
    fn build(&self, app: &mut App) {
        let state = Arc::new(Mutex::new(State::default()));
        let conn = Conn::new(ConnMode::ReadOnly);
        //let conn = Conn::new(ConnMode::ReadWrite); // uncomment to work "for real"
        app.world.insert_resource(BulbState {
            inner: state.clone(),
        });

        thread::spawn(move || {
            run(state, conn).unwrap_or_else(|e| eprintln!("Background task failed: {}", e));
        });
    }
}
