use core::fmt::Debug;

use embedded_nrf24l01 as nrf;
use nrf::{Device, Configuration};

pub fn setup_nrf<D>(nrf: &mut impl Configuration<Inner = D>, addr: &[u8])
where D: Device,
      D::Error: Debug
{
    nrf.set_frequency(100).expect("Failed to set frequency");
    nrf.set_auto_retransmit(0, 0).expect("Failed to set auto retransmit");
    // nrf.set_crc(Some(CrcMode::TwoBytes)).unwrap();
    // nrf.set_rf(DataRate::R250Kbps, 1).unwrap();
    nrf
        .set_auto_ack(&[true, false, false, false, false, false])
        .expect("Failed to set auto ack");
    nrf
        .set_pipes_rx_enable(&[true, false, false, false, false, false])
        .expect("Failed to set rx enable");
    nrf
        .set_pipes_rx_lengths(&[None, None, None, None, None, None])
        .expect("Failed to set pipes rx lengths");
    nrf.set_tx_addr(&addr).expect("Failed to set tx addr");
    nrf.set_rx_addr(0, &addr).expect("Failed to set rx addr");
    nrf.flush_rx().expect("Failed to flush rx");
    nrf.flush_tx().expect("Failed to flush tx");
}
