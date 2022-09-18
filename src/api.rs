use std::io::Read;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::mpsc::TryRecvError;

use ble::serialize_bulb_states;
use rouille::{RequestBody, Request};

use std::sync::Mutex;
use std::sync::Arc;

use anyhow::Result;
use crate::lib::{Bulb, BulbCmd, BulbStates, BulbState};
use crate::lib;

fn parse_body(request: &Request) -> Result<BulbStates> {
        if let Some(body) = request.data().as_mut() {
            let mut buf = Vec::new();
            body.read_to_end(&mut buf)?;
            let s = String::from_utf8(buf)?;
            let parsed: BulbStates = serde_json::from_str(&s)?;
            Ok(parsed)
        }
        else {
            Err(anyhow::format_err!("no data in request body"))
        }
}

pub fn run(
    bulb_states_unlocked: Arc<Mutex<BulbStates>>,
    bulb_cmd_tx: Sender<BulbCmd>
    ) {
    //let mut bulb_states = bulb_states_rx.recv().unwrap();
    
    // rouille spins up multiple threads to handle requests
    // docs call this behavior "synchronous", as opposed to using a green thread runtime
    // which would be called "asynchronous". I disagree with this naming, since 
    // one would reasonably expect a synchronous library not to start any threads at all.
    // Alas, I will wrap my queues in mutexes for now, maybe one day I'll find a way to make
    // rouille single-thread?
    
    ///let bulb_states_rx_unlocked = Mutex::new(bulb_states_rx);

    let bulb_cmd_tx_unlocked = std::sync::Arc::new(Mutex::new(bulb_cmd_tx));

    rouille::start_server("localhost:8000", move |request| {
    //let bulb_states_rx = bulb_states_rx_unlocked.lock().unwrap();
    let bulb_cmd_tx = bulb_cmd_tx_unlocked.lock().unwrap();

    if request.url().ends_with("/api") {
        if request.method() == "POST" {
            if let Ok(desired_bulb_states) = parse_body(request) {

                let bulb_states = bulb_states_unlocked.lock().unwrap();

                for k in desired_bulb_states.keys() {
                    let maybe_known_bulb = bulb_states.get(k);
                    if let Some(known_bulb) = maybe_known_bulb {
                        let cmds = known_bulb
                            .diff(desired_bulb_states.get(k).unwrap())
                            .into_iter()
                            .map(|cmd| lib::BulbCmd{cmd, bulb_id: k.clone()});
                        for cmd in cmds {
                            bulb_cmd_tx.send(cmd).expect("tx failed, something very wrong");
                        }
                    } else {
                        log::warn!("Requested unknown bulb id {}", k);
                    }
                }
                rouille::Response::empty_204()
            } else {
                log::warn!("malformed request");
                rouille::Response::empty_406()

            }
        } else if request.method() == "GET" {
            use std::ops::Deref;
            let bulb_states = bulb_states_unlocked.lock().unwrap();
            rouille::Response::json(bulb_states.deref())
        } else {
            rouille::Response::empty_400()
        }
    }
    else {
        let response = rouille::match_assets(&request, "static");
        if response.is_success() {
            response
        } else {
            rouille::Response::empty_404()
        }
    }



});
}
