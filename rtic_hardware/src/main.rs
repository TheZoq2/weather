#![no_main]
#![no_std]

use panic_semihosting as _;
use rtic::app;
use cortex_m_semihosting::hprintln;

use nb::block;

use embedded_hal::digital::v2::OutputPin;

use cortex_m::asm;

use stm32f1xx_hal::{
    prelude::*,
    gpio::{
        gpioa::{PA4, PA3, PA5, PA6, PA7},
        Output,
        PushPull,
        Alternate,
        Input,
        Floating,
    },
    delay::Delay,
    pac::SPI1,
    spi::{Spi, Spi1NoRemap},
};

use embedded_nrf24l01 as nrf;
use nrf::{StandbyMode, NRF24L01};
use nrf::Configuration;

// pub type EspType = Esp8266<Tx<USART1>, Rx<USART1>, LongTimer<TIM2>, PA8<Output<PushPull>>>;
pub type NrfType = NRF24L01<
    core::convert::Infallible,
    PA4<Output<PushPull>>,
    PA3<Output<PushPull>>,
    Spi<
        SPI1,
        Spi1NoRemap,
        (PA5<Alternate<PushPull>>, PA6<Input<Floating>>, PA7<Alternate<PushPull>>)
    >
>;

#[app(device = stm32f1xx_hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        nrf: StandbyMode<NrfType>,
    }

    #[init]
    fn init(ctx: init::Context) -> init::LateResources {
        let dp = ctx.device;
        let cp = ctx.core;

        let mut flash = dp.FLASH.constrain();
        let mut rcc = dp.RCC.constrain();
        let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
        let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);
        let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

        let mut status_led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
        status_led.set_low();

        let clocks = rcc.cfgr.freeze(&mut flash.acr);

        let mut delay = Delay::new(cp.SYST, clocks);

        let ce = gpioa.pa4.into_push_pull_output(&mut gpioa.crl);
        let csn = gpioa.pa3.into_push_pull_output(&mut gpioa.crl);
        let pins = (
            gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl),
            gpioa.pa6.into_floating_input(&mut gpioa.crl),
            gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl),
        );

        let spi = Spi::spi1(
            dp.SPI1,
            pins,
            &mut afio.mapr,
            embedded_nrf24l01::setup::spi_mode(),
            embedded_nrf24l01::setup::clock_mhz().mhz(),
            clocks,
            &mut rcc.apb2
        );

        let mut nrf = NRF24L01::new(ce, csn, spi).unwrap();
        hprintln!("AutoAck {:?}", nrf.get_auto_ack().unwrap()).unwrap();
        hprintln!("Register {:?}", nrf.get_address_width().unwrap()).unwrap();
        hprintln!("Frequency {:?}", nrf.get_frequency().unwrap()).unwrap();

        let addr: [u8; 5] = [0x22, 0x22, 0x22, 0x22, 0x22];

        nrf.set_frequency(100).unwrap();
        nrf.set_auto_retransmit(0, 0).unwrap();
        // nrf.set_crc(Some(CrcMode::TwoBytes)).unwrap();
        // nrf.set_rf(DataRate::R250Kbps, 1).unwrap();
        nrf
            .set_auto_ack(&[true, false, false, false, false, false])
            .unwrap();
        nrf
            .set_pipes_rx_enable(&[true, false, false, false, false, false])
            .unwrap();
        nrf
            .set_pipes_rx_lengths(&[None, None, None, None, None, None])
            .unwrap();
        nrf.set_tx_addr(&addr).unwrap();
        nrf.set_rx_addr(0, &addr).unwrap();
        nrf.flush_rx().unwrap();
        nrf.flush_tx().unwrap();

        delay.delay_ms(10 as u16);

        // nrf.set_tx_addr(b"00001").expect("failed to set addres");

        let nrf = {
            let mut nrf = nrf.tx().expect("Failed to go to TX mode");

            delay.delay_ms(130 as u16);

            nrf.send(b"hello, world")
                .expect("Failed to send hello world");
            let success = block!(nrf.poll_send()).expect("Poll error");
            if !success {
                hprintln!("Failed to send message")
                    .unwrap();
                loop {continue;}
            }
            nrf.standby()
                .expect("Failed to go back to standby mode")
        };

        status_led.set_high();


        init::LateResources {
            nrf
        }
    }

    #[idle()]
    fn idle(_ctx: idle::Context) -> ! {
        loop {
            asm::wfi()
        }
    }
};
