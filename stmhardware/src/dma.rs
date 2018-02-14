use f3::hal::stm32f30x::{self, dma1};

fn setup_dma(
        data: &'static mut [u8],
        cmar: dma1::CMAR4,
        ccr: dma1::CMAR4,
        cndt: dma1::CNDTR4,
        cpar: dma1::CPAR4
    )
{
    // Write USART_RDR reg address to DMA
    // Write output memory address to DMA
    // Configure total number of bytes
    // Configure channel priority
    // Configure interrupts to trigger if sending stopped or buffer is full
    // Activate channel
}
