use arrayvec::ArrayString;
use itoa;
use error;

use core::fmt::Write;

const MESSAGE_MAX_LEN: usize = 32; // Chosen arbitrarily

// TODO: Handle errors by replacing `push` to `try_push`
pub fn encode_i32(
    name: &str,
    val: i32,
    buffer: &mut ArrayString<[u8; MESSAGE_MAX_LEN]>
) -> Result<(), error::EncodingError>
{
    let mut val_str = ArrayString::<[u8; MESSAGE_MAX_LEN]>::new();

    itoa::fmt(&mut val_str, val)?;

    if let Err(e) = buffer.try_push_str(name) {Err(e.simplify())?};
    if let Err(e) = buffer.try_push(':') {Err(e.simplify())?};
    if let Err(e) = buffer.try_push_str(&val_str) {Err(e.simplify())?};

    // write!(buffer, "{}:{}", name, val)?;
    Ok(())
}

// TODO: Handle errors by replacing `push` to `try_push`
pub fn encode_f32(
    name: &str,
    val: f32,
    buffer: &mut ArrayString<[u8; MESSAGE_MAX_LEN]>
) -> Result<(), error::EncodingError>
{
    write!(buffer, "{}:{}", name, val)?;

    Ok(())
}
