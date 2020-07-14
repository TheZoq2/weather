#![no_std]

use serde::{Serialize, Deserialize};

use bmp085_driver as bno;

pub mod nrf;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Pascal(i32);
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DeciCelcius(i32);

impl From<bno::DeciCelcius> for DeciCelcius {
    fn from(other: bno::DeciCelcius) -> Self {
        DeciCelcius(other)
    }
}
impl From<bno::Pascal> for Pascal {
    fn from(other: bno::Pascal) -> Self {
        Pascal(other)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SensorReading {
    Pressure(Pascal),
    Temperature(DeciCelcius),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Message<'a> {
    Reading(SensorReading),
    Error(&'a str)
}
