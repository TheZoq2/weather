use arrayvec::ArrayString;

use core::fmt::Write;

const MESSAGE_MAX_LEN: usize = 16; // Chosen arbitrarily

pub fn encode_u16(
    name: &str,
    val: f32,
    buffer: &mut ArrayString<[u8; MESSAGE_MAX_LEN]>
) -> Result<(), ()>
{
    write!(buffer, "{}:{}", name, val);
    Ok(())
}
