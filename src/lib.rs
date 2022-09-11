use std::collections::{BTreeSet, HashMap};
use std::iter::FromIterator;

use anyhow::Result;
use btleplug::api::{BDAddr, Central, Characteristic, Manager as _, Peripheral as _, WriteType};
use btleplug::platform::{Adapter, Manager, Peripheral};

use uuid::Uuid;
//const CHAR_UUID_STATE_ON_OFF: Uuid = Uuid::from_u128(0x932c32bd_0002_47a2_835a_a8d455b859dd);
//const CHAR_UUID_COLOR_AND_BRIGHTNESS: Uuid = Uuid::from_u128(0x932c32bd_0007_47a2_835a_a8d455b859dd);
const CHAR_UUID_HUE: Uuid = Uuid::from_u128(0x932c32bd_0005_47a2_835a_a8d455b859dd);
const CHAR_UUID_BRIGHTNESS: Uuid = Uuid::from_u128(0x932c32bd_0003_47a2_835a_a8d455b859dd);

fn linterp(x: f32, a: f32, b: f32) -> f32 {
    a * (1.0 - x) + x * b
}

pub struct Bulb {
    char_brightness: Characteristic,
    char_hue: Characteristic,
    peripheral: Peripheral,
}

impl Bulb {
    pub async fn new(adapter: &Adapter, addr: &BDAddr) -> Result<Self> {
        let peripheral = Self::find_bulb(adapter, addr).await?;

        let char_brightness = peripheral
            .characteristics()
            .iter()
            //.map(|c| { log::info!("{:?}", c); c })
            .find(|c| c.uuid == CHAR_UUID_BRIGHTNESS)
            .expect("failed to find brightness characteristic")
            .to_owned();
        let char_hue = peripheral
            .characteristics()
            .iter()
            .find(|c| c.uuid == CHAR_UUID_HUE)
            .expect("failed to find hue characteristic")
            .to_owned();
        Ok(Self {
            peripheral,
            char_brightness,
            char_hue,
        })
    }

    async fn find_bulb(adapter: &Adapter, addr: &BDAddr) -> Result<Peripheral> {
        for p in adapter.peripherals().await.unwrap() {
            if p.properties().await.unwrap().unwrap().address == *addr {
                p.discover_services().await?;
                return Ok(p);
            }
        }
        Err(anyhow::format_err!("couldn't find bulb with addr {}", addr))
    }

    pub async fn set_brightness(&mut self, v: f32) -> Result<()> {
        let v_inter = linterp(v, 1.0, 247.0);
        log::info!("set brightness={}", v_inter);
        self.peripheral
            .write(
                &self.char_brightness,
                &[v_inter as u8],
                WriteType::WithoutResponse,
            )
            .await?;
        Ok(())
    }

    pub async fn set_hue(&mut self, h: palette::Yxy) -> Result<()> {
        let x_inter = linterp(h.x as f32, 0.0, 255.0);
        let y_inter = linterp(h.y, 0.0, 255.0);
        log::info!("set color={}, {}", x_inter, y_inter);
        self.peripheral
            .write(
                &self.char_hue,
                &[0, x_inter as u8, 0, y_inter as u8],
                WriteType::WithResponse,
            )
            .await?;
        Ok(())
    }
}
