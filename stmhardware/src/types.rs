use stm32f103xx_hal::gpio::{Input, Floating};
use stm32f103xx_hal::gpio::gpioa::{PA0};
use stm32f103xx_hal::serial::{Rx, Tx};
use stm32f103xx_hal::timer::Timer;
use stm32f103xx::{TIM2, TIM3, TIM4, USART1};

use anemometer;
use esp8266;
use dhtxx;

pub type AnemometerType = anemometer::Anemometer<PA0<Input<Floating>>, Timer<TIM3>>;

pub type EspType = esp8266::Esp8266<Tx<USART1>, Rx<USART1>, Timer<TIM2>>;

pub type DhtType = dhtxx::Dhtxx;
