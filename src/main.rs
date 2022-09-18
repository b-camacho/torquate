mod lib;
mod api;

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::mpsc::channel;
use std::time::Duration;

use anyhow::Result;
use ble::BulbCmdType;
use btleplug::api::{BDAddr, Central, Manager as _, Peripheral as _, WriteType};
use btleplug::platform::{Adapter, Manager, Peripheral};
use json::JsonValue;
use palette::convert::TryIntoColor;
use std::error::Error;
use uuid::Uuid;
use std::sync::Mutex;
use std::sync::Arc;

fn parse_bulb_list(jv: JsonValue) -> HashMap<String, BDAddr> {
    jv.entries()
        .map(|(key, v)| {
            (
                key.to_owned(),
                BDAddr::from_str(v.as_str().unwrap()).unwrap(),
            )
        })
        .collect::<HashMap<_, _>>()
}

fn main() -> Result<(), Box<dyn Error>> {

    env_logger::init();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let manager = rt.block_on(Manager::new()).unwrap();

    // get the first bluetooth adapter
    let central = rt.block_on(manager
        .adapters())
        .expect("Unable to fetch adapter list.")
        .into_iter()
        .nth(0)
        .expect("Unable to find adapters.");

    let bulb_json = json::parse(
        r#"
{
    "nightstand": "EB:B0:58:AC:E7:C7"
}
"#,
    )
    .unwrap();

    let bulb_map = parse_bulb_list(bulb_json);
    let mut bulbs = bulb_map.into_iter()
        .map(|(id, bdaddr)| 
            (id, rt.block_on(lib::Bulb::new(&central, &bdaddr)).expect("failed to construct bulb"))
        )
        .collect::<HashMap<_, _>>();

    let bulb_states: lib::BulbStates = bulbs.iter_mut()
        .map(|(id, bulb)| (id.clone(), rt.block_on(bulb.get_state()).unwrap()))
        .collect::<lib::BulbStates>();

    log::info!("{bulb_states:?}");


    std::thread::scope(move |s| {
        //let (bulb_states_tx, bulb_states_rx) = channel::<lib::BulbStates>();
        let (bulb_cmd_tx, bulb_cmd_rx) = channel::<lib::BulbCmd>();
        let bulb_states_unlocked = Arc::new(Mutex::new(bulb_states));
        let bulb_states_unlocked_clone = bulb_states_unlocked.clone();
        let bulb_states_unlocked_clone_2 = bulb_states_unlocked.clone();


        let bulb_cmd_tx_clone = bulb_cmd_tx.clone();
        s.spawn(move || {
            api::run(bulb_states_unlocked.clone(), bulb_cmd_tx_clone);
        });

        s.spawn(move || {
            let bulb_states_unlocked = bulb_states_unlocked_clone;
            loop {
                let mut new_bulb_cmd = bulb_cmd_rx.recv().unwrap();
                let bulb_id = new_bulb_cmd.bulb_id;
                let bulb = bulbs.get_mut(&bulb_id).unwrap();
                use palette::IntoColor;
                let cmd_future = match new_bulb_cmd.cmd {
                    lib::BulbCmdType::SetBrightness(b) => rt.block_on(bulb.set_brightness(b)),
                    lib::BulbCmdType::SetHue(h) =>
                        rt.block_on(bulb.set_hue(
                                palette::Srgb::new(h[0], h[1], h[2])
                                .into_color())),
                    lib::BulbCmdType::GetState => {
                        let mut bulb_states = bulb_states_unlocked.lock().unwrap();
                        let mut bulb_state = bulb_states.get_mut(&bulb_id).unwrap();
                        let updated_bulb_state = rt.block_on(bulb.get_state()).unwrap();
                        bulb_state.rgb = updated_bulb_state.rgb;
                        bulb_state.brightness = updated_bulb_state.brightness;
                        Ok(())
                    }
                };
            };
        });

        s.spawn(move || {
            let bulb_states_unlocked = bulb_states_unlocked_clone_2;
            loop {
                {
                    let bulb_states = bulb_states_unlocked.lock().unwrap();
                    for k in bulb_states.keys() {
                        bulb_cmd_tx.send(lib::BulbCmd{bulb_id: k.to_owned(), cmd: lib::BulbCmdType::GetState
                        });
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            };
        });


    });

    //let mut bulb = lib::Bulb::new(&central, &bulb_map["nightstand"])
    //    .await
    //    .unwrap();


    //use palette::IntoColor;
    //let yxy_color: palette::Yxy = palette::Srgb::new(0.0, 1.0, 0.0).into_color();
    //bulb.set_hue(
    //    yxy_color
    //    ).await?;
    //
    //std::thread::sleep(Duration::from_secs(1));

    //bulb.set_brightness(0.0).await?;
    //std::thread::sleep(Duration::from_secs(1));
    //bulb.set_brightness(1.0).await?;
    //std::thread::sleep(Duration::from_secs(1));
    //bulb.set_brightness(0.0).await?;


    return Ok(());
}
