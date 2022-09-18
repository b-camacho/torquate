use std::collections::{BTreeSet, HashMap};
use palette::IntoColor;
use serde::{Deserialize, Serialize};
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

#[derive(Debug)]
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

    pub async fn get_state(&mut self) -> Result<BulbState> {
        let hue = self.peripheral.read(&self.char_hue).await?;
        let hue_rgb: palette::Srgb = palette::Yxy::new((hue[1] as f32) / 255.0, (hue[3] as f32) / 255.0, 1.0).into_color();
        let brightness = self.peripheral.read(&self.char_brightness).await?;
        Ok(BulbState {
            rgb: [hue_rgb.red, hue_rgb.green, hue_rgb.blue],
            brightness: brightness[0] as f32 / 255.0
        })
    }
}

pub struct BulbCmd {
    pub bulb_id: String,
    pub cmd: BulbCmdType,
}

pub enum BulbCmdType {
    SetBrightness(f32),
    SetHue([f32; 3]),
    GetState,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulbState {
    pub rgb: [f32; 3],
    pub brightness: f32,
}

impl BulbState {
    pub fn diff(&self, other: &BulbState) -> Vec<BulbCmdType> {
        let mut out = vec![];

        if self.rgb != other.rgb {
            out.push(BulbCmdType::SetHue(other.rgb))
        }
        
        if self.brightness != other.brightness {
            out.push(BulbCmdType::SetBrightness(other.brightness))
        }

        out
    }
}

pub type BulbStates = std::collections::HashMap<String, BulbState>;

pub fn serialize_bulb_states(bulb_states: &BulbStates) -> String {
    "{}".to_string()
}

