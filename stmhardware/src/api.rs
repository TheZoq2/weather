#[repr(u8)]
#[derive(Clone)]
pub enum ReadingType {
    Temperature,
    Pressure,
    WindSpeedRaw
}

struct Reading {
    reading_type: ReadingType,
    value: i32
}

impl Reading {
    pub fn encode(&self) -> [u8;5] {
        let mut result = [0;5];
        result[0] = self.reading_type.clone() as u8;
        result[1] = (self.value >> 24) as u8;
        result[2] = (self.value >> 16) as u8;
        result[3] = (self.value >> 8) as u8;
        result[4] = (self.value) as u8;
        result
    }

}
