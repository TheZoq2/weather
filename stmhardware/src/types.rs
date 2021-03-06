use stm32f1xx_hal::gpio::{Input, Output, Floating, PushPull};
use stm32f1xx_hal::gpio::gpioa::{PA1, PA8};
use stm32f1xx_hal::serial::{Rx, Tx};
use stm32f1xx_hal::timer::Timer;
use stm32f1xx_hal::stm32::{TIM2, TIM3, TIM4, USART1};
use void::Void;

use crate::anemometer;
use crate::esp8266;
use crate::serial;
use crate::dhtxx;


pub type AnemometerType = anemometer::Anemometer<PA1<Input<Floating>>, Timer<TIM3>>;

pub type SerialReadError = serial::Error<stm32f1xx_hal::serial::Error>;
pub type SerialWriteError = Void;

pub type EspRxType = Rx<USART1>;
pub type EspTxType = Tx<USART1>;
pub type EspType = esp8266::Esp8266<EspTxType, EspRxType, Timer<TIM2>, PA8<Output<PushPull>>>;

pub type DhtType = dhtxx::Dhtxx;
