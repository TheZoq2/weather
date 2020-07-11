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
    pac::{PWR, EXTI, SPI1},
    spi::{Spi, Spi1NoRemap},
    rtc::Rtc,
};

use embedded_nrf24l01 as nrf;
use nrf::{StandbyMode, NRF24L01};

use common::nrf::setup_nrf;

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
        nrf: Option<StandbyMode<NrfType>>,
        rtc: Rtc,
        // Required for sleep
        exti: EXTI,
        scb: cortex_m::peripheral::SCB,
        pwr: PWR,

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
        let mut pwr = dp.PWR;


        let mut backup_domain = rcc.bkp.constrain(dp.BKP, &mut rcc.apb1, &mut pwr);

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

        let addr: [u8; 5] = [0x22, 0x22, 0x22, 0x22, 0x22];

        setup_nrf(&mut nrf, &addr);

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

        status_led.set_high().unwrap();

        let mut rtc = Rtc::rtc(dp.RTC, &mut backup_domain);
        rtc.select_frequency(1.hz());
        rtc.listen_alarm();

        init::LateResources {
            nrf: Some(nrf),
            rtc,
            exti: dp.EXTI,
            scb: cp.SCB,
            pwr,
        }
    }

    #[idle(resources=[rtc, pwr, scb, exti])]
    fn idle(ctx: idle::Context) -> ! {
        let mut r = ctx.resources;
        loop {
            r.rtc.lock(|rtc| {
                rtc.set_time(0);
                rtc.set_alarm(2);
                rtc.clear_alarm_flag();
            });

            r.exti.lock(|exti| {
                // Enable RTCAlarm event
                exti.imr.modify(|_r, w| w.mr17().set_bit());
                // Maybe set rising or falling edge as well
                exti.rtsr.modify(|_r, w| w.tr17().set_bit());
            });

            let scb = &mut r.scb;
            let pwr = &mut r.pwr;

            stop_mode(scb, pwr);
            // asm::wfi();
        }
    }

    #[task(binds = RTCALARM, resources=[nrf, rtc, exti])]
    fn on_rtc_alarm(ctx: on_rtc_alarm::Context) {
        let r = ctx.resources;

        // Clear the pending bit
        r.exti.pr.modify(|_r, w| w.pr17().set_bit());

        // TODO: Remove once runnign on HW
        // hprintln!("T");

        // clear the alarm
        r.rtc.set_time(0);
        r.rtc.clear_alarm_flag();

        // Transmit a ping
        // NOTE: Safe unwrap because we'll make sure to put this back
        // NOTE: Actually unsafe, if we take the NRF out elsewhere, in an interrupt,
        // we'll have a fun issue to debug
        let nrf = r.nrf.take().unwrap();

        let mut nrf = nrf.tx().expect("Failed to go to TX mode");

        nrf.send(b"Timer")
            .expect("Failed to send hello world");
        let _success = block!(nrf.poll_send()).expect("Poll error");
        // if !success {}

        *r.nrf = Some(nrf.standby().expect("Failed to go back to standby mode"));
    }
};


fn stop_mode(
    system_control_block: &mut cortex_m::peripheral::SCB,
    pwr: &mut PWR,
) {
    // Set SLEEPDEEP in cortex-m3 system control register
    system_control_block.set_sleepdeep();

    // // Clear PDDS bit in PWR_CR to enable stop mode
    // // Set voltage regulator mode using LDPS in PWR_CR
    pwr.cr.modify(|_r, w| {
        // Enable stop mode
        w.pdds().clear_bit()
        // Voltage regulators to low power mode
         .lpds().set_bit()
    });

    // Call asm::wfi() or asm::wfe()
    // asm::bkpt();
    asm::wfi();
    asm::nop();
    // asm::bkpt();
}
