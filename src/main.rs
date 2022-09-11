mod lib;

use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;

use anyhow::Result;
use btleplug::api::{BDAddr, Central, Manager as _, Peripheral as _, WriteType};
use btleplug::platform::{Adapter, Manager, Peripheral};
use json::JsonValue;
use palette::convert::TryIntoColor;
use std::error::Error;
use uuid::Uuid;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let manager = Manager::new().await.unwrap();

    // get the first bluetooth adapter
    let central = manager
        .adapters()
        .await
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

    let mut bulb = lib::Bulb::new(&central, &bulb_map["nightstand"])
        .await
        .unwrap();

    use palette::IntoColor;
    let yxy_color: palette::Yxy = palette::Srgb::new(0.0, 1.0, 0.0).into_color();
    bulb.set_hue(
        yxy_color
        ).await?;
    
    std::thread::sleep(Duration::from_secs(1));

    bulb.set_brightness(0.0).await?;
    std::thread::sleep(Duration::from_secs(1));
    bulb.set_brightness(1.0).await?;
    std::thread::sleep(Duration::from_secs(1));
    bulb.set_brightness(0.0).await?;


    return Ok(());
}
