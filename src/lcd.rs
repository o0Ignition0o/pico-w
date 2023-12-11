use embassy_rp::i2c::{I2c, Instance, Mode};

pub struct Lcd<'d, T: Instance, M: Mode> {
    i2c: I2c<'d, T, M>,
    address: u8,
}

impl<'d, T: Instance, M: Mode> Lcd<'d, T, M> {
    pub fn new(i2c: I2c<'d, T, M>, address: u8) -> Self {
        Self { i2c, address }
    }
}
