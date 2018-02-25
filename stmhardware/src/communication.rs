use arrayvec::ArrayString;

use core::fmt::Write;

const MESSAGE_MAX_LEN: usize = 32; // Chosen arbitrarily

pub fn encode_f32(
    name: &str,
    val: i32,
    buffer: &mut ArrayString<[u8; MESSAGE_MAX_LEN]>
) -> Result<(), ::core::fmt::Error>
{
    write!(buffer, "{}:{}", name, val)?;
    Ok(())
}
