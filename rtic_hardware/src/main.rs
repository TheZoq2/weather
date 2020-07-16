#![no_main]
#![no_std]

mod latest_queue;

use panic_semihosting as _;
use rtic::app;
use cortex_m_semihosting::hprintln;

use nb::block;

use embedded_hal::digital::v2::OutputPin;

use cortex_m::asm;

use stm32f1xx_hal::{
    prelude::*,
    gpio::{
        gpioa::{PA0, PA1, PA4, PA3, PA5, PA6, PA7},
        gpiob::{PB6, PB7},
        Output,
        PushPull,
        Alternate,
        Input,
        Floating,
        PullUp,
        ExtiPin,
        Edge,
        OpenDrain,
        Analog,
    },
    adc::Adc,
    delay::Delay,
    pac::{PWR, EXTI, SPI1, I2C1, ADC1},
    spi::{self, Spi, Spi1NoRemap},
    rtc::Rtc,
    i2c::{self, BlockingI2c},
    rcc::{self, Clocks},
};

use embedded_nrf24l01 as nrf;
use nrf::{StandbyMode, NRF24L01};

use bmp085_driver as bmp;
use bmp::Bmp085;

use heapless::consts;
use heapless::{Vec, HistoryBuffer, ArrayLength};

use common::nrf::setup_nrf;
use common::{Message, SensorReading};

use core::fmt::Write;

const SLEEP_DURATION: u32 = 10;

#[derive(Debug)]
pub enum Error {
    TransmitFailure,
    BmpReadError(i2c::Error),
    NrfModeError(nrf::Error<spi::Error>),
    NrfTxError(nrf::Error<spi::Error>),
    NrfPollError(nrf::Error<spi::Error>),
    EncodingError(postcard::Error),
    FmtErr(core::fmt::Error),
}

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

pub type BmpType = Bmp085<
    BlockingI2c<I2C1, (PB6<Alternate<OpenDrain>>, PB7<Alternate<OpenDrain>>)>,
    Delay,
>;

#[app(device = stm32f1xx_hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        nrf: Option<StandbyMode<NrfType>>,
        bmp: BmpType,
        rtc: Rtc,
        rainmeter_pin: PA0<Input<PullUp>>,
        rainmeter_counter: usize,
        // Required for sleep
        exti: EXTI,
        scb: cortex_m::peripheral::SCB,
        pwr: PWR,
        errors: latest_queue::LatestQueue<Error, consts::U10>,
        // Required for the ADC
        adc: Option<ADC1>,
        apb2: rcc::APB2,
        adc_pin: PA1<Analog>,
        clocks: Clocks,
    }

    #[init]
    fn init(ctx: init::Context) -> init::LateResources {
        let dp = ctx.device;
        let cp = ctx.core;

        let mut flash = dp.FLASH.constrain();
        let mut rcc = dp.RCC.constrain();
        let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
        let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);
        let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);
        let mut afio = dp.AFIO.constrain(&mut rcc.apb2);
        let mut pwr = dp.PWR;
        let mut exti = dp.EXTI;


        let mut backup_domain = rcc.bkp.constrain(dp.BKP, &mut rcc.apb1, &mut pwr);

        let mut status_led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
        status_led.set_low().unwrap();

        let clocks = rcc.cfgr
            .freeze(&mut flash.acr);


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


        let mut rtc = Rtc::rtc(dp.RTC, &mut backup_domain);
        rtc.select_frequency(1.hz());
        rtc.listen_alarm();

        // Initialise rainmeter
        let mut rainmeter_pin = gpioa.pa0.into_pull_up_input(&mut gpioa.crl);
        rainmeter_pin.make_interrupt_source(&mut afio);
        rainmeter_pin.trigger_on_edge(&mut exti, Edge::FALLING);
        rainmeter_pin.enable_interrupt(&mut exti);

        let bmp_pins = (
                gpiob.pb6.into_alternate_open_drain(&mut gpiob.crl),
                gpiob.pb7.into_alternate_open_drain(&mut gpiob.crl),
            );
        let i2c = BlockingI2c::i2c1(
                dp.I2C1,
                bmp_pins,
                &mut afio.mapr,
                i2c::Mode::Standard{frequency: 100_000.hz()},
                clocks,
                &mut rcc.apb1,
                100,
                1,
                100,
                100,
            );

        let bmp = bmp::Bmp085::new(i2c, delay, bmp::Oversampling::Standard)
            .unwrap();

        status_led.set_high().unwrap();

        init::LateResources {
            nrf: Some(nrf),
            bmp,
            rainmeter_pin,
            rainmeter_counter: 0,
            rtc,
            exti,
            scb: cp.SCB,
            pwr,
            errors: latest_queue::LatestQueue::new(),
            adc: Some(dp.ADC1),
            apb2: rcc.apb2,
            adc_pin: gpioa.pa1.into_analog(&mut gpioa.crl),
            clocks,
        }
    }

    #[idle(resources=[rtc, pwr, scb, exti])]
    fn idle(ctx: idle::Context) -> ! {
        let mut r = ctx.resources;
        loop {
            r.rtc.lock(|rtc| {
                rtc.set_time(0);
                rtc.set_alarm(SLEEP_DURATION);
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

    #[task(binds = RTCALARM, resources=[
        nrf,
        rtc,
        exti,
        rainmeter_counter,
        bmp,
        errors,
        adc,
        apb2,
        clocks,
        adc_pin,
    ])]
    fn on_rtc_alarm(ctx: on_rtc_alarm::Context) {
        let r = ctx.resources;

        // clear the alarm
        r.rtc.set_time(0);
        r.rtc.clear_alarm_flag();
        // Clear the pending bit
        r.exti.pr.modify(|_r, w| w.pr17().set_bit());

        let mut messages: Vec<_, consts::U10> = Vec::new();


        // NOTE: This block is kind of strange. It is *not* blocking for read, it is blockign for
        // the i2c error, which in reality *should* never be WouldBlock
        match block!(r.bmp.read()) {
            Ok(val) => {
                messages.push(Message::Reading(
                    SensorReading::Temperature(val.temperature.into())
                )).unwrap();
                messages.push(Message::Reading(
                    SensorReading::Pressure(val.pressure.into())
                )).unwrap();
            },
            Err(e) => r.errors.push(Error::BmpReadError(e))
        };

        let (battery, adc) = measure_battery(
            r.adc.take().unwrap(),
            r.adc_pin,
            r.apb2,
            *r.clocks
        );
        *r.adc = Some(adc);
        messages.push(Message::Reading(SensorReading::Battery(battery)))
            .unwrap();


        // Transmit the messages
        // NOTE: Safe unwrap because we'll make sure to put this back
        // NOTE: Actually unsafe, if we take the NRF out elsewhere, in an interrupt,
        // we'll have a fun issue to debug
        let nrf = r.nrf.take().unwrap();

        match nrf.tx() {
            Ok(mut nrf) => {
                for message in messages {
                    // This error will most likely be unrecoverable which is why
                    // it is not stored
                    try_or_log(send_message(&message, &mut nrf), r.errors, |x| x);
                }

                let mut err_transmit_err = None;
                for error in &r.errors.inner {
                    // This error will most likely be unrecoverable which is why
                    // it is not stored
                    let mut s = heapless::String::<consts::U64>::new();
                    if let Err(e) = write!(s, "{:?}", error) {
                        err_transmit_err = Some(Error::FmtErr(e))
                    }
                    if let Err(e) = send_message(&Message::Error(&s), &mut nrf) {
                        err_transmit_err = Some(e);
                        break
                    }
                };

                if let Some(e) = err_transmit_err {
                    r.errors.push(e)
                }
                else {
                    // Since we transmitted all errors without failure, we can
                    // clear the list now
                    r.errors.clear()
                }

                // Recovering this error will be very difficult because we can not store a standby
                // mode device
                *r.nrf = Some(
                    nrf.standby().expect("Failed to go back to standby mode")
                )
            }
            Err((standby, err)) => {
                r.errors.push(Error::NrfModeError(err));
                *r.nrf = Some(StandbyMode::power_up(standby).expect("Failed to power device back up"))
            }
        }
    }

    #[task(binds=EXTI0, resources=[rainmeter_pin, rainmeter_counter])]
    fn on_rainmeter_toggle(ctx: on_rainmeter_toggle::Context) {
        ctx.resources.rainmeter_pin.clear_interrupt_pending_bit();
        *ctx.resources.rainmeter_counter += 1;
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


fn try_or_log<T, E>(
    result: Result<T, E>,
    errors: &mut latest_queue::LatestQueue<Error, consts::U10>,
    descriptor: impl Fn(E) -> Error,
) -> Option<T> {
    match result {
        Ok(val) => Some(val),
        Err(e) => {
            errors.push(descriptor(e));
            None
        }
    }
}


fn send_message(message: &Message, nrf: &mut nrf::TxMode<NrfType>)
    -> Result<(), Error>
{
    let bytes = postcard::to_vec::<consts::U128, _>(message)
        .map_err(Error::EncodingError)?;

    nrf.send(&bytes)
        .map_err(Error::NrfTxError)?;

    let success = block!(nrf.poll_send())
        .map_err(Error::NrfPollError)?;

    if !success {
        Err(Error::TransmitFailure)
    }
    else {
        Ok(())
    }
}


fn measure_battery(
    adc: ADC1,
    pin: &mut PA1<Analog>,
    apb2: &mut rcc::APB2,
    clocks: Clocks
) -> (f32, ADC1) {
    let mut adc = Adc::adc1(adc, apb2, clocks);

    let reading: u16 = block!(adc.read(pin)).unwrap();
    let value = reading as f32 / adc.max_sample() as f32;

    (value, adc.release(apb2))
}
