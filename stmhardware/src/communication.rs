use arrayvec::ArrayString;
use itoa;
use error::Error;

use core::fmt::Write;

const MESSAGE_MAX_LEN: usize = 32; // Chosen arbitrarily

// TODO: Handle errors by replacing `push` to `try_push`
pub fn encode_i32(
    name: &str,
    val: i32,
    buffer: &mut ArrayString<[u8; MESSAGE_MAX_LEN]>
) -> Result<(), Error>
{
    let mut val_str = ArrayString::<[u8; MESSAGE_MAX_LEN]>::new();

    itoa::fmt(&mut val_str, val)?;

    buffer.try_push_str(name)?;
    buffer.try_push(':')?;
    buffer.try_push_str(&val_str)?;

    // write!(buffer, "{}:{}", name, val)?;
    Ok(())
}

// TODO: Handle errors by replacing `push` to `try_push`
pub fn encode_f32(
    name: &str,
    val: f32,
    buffer: &mut ArrayString<[u8; MESSAGE_MAX_LEN]>
) -> Result<(), Error>
{
    write!(buffer, "{}:{}", name, val)?;

    Ok(())
}
