use embassy_rp::i2c::Error;
use embassy_rp::i2c::{I2c, Instance, Mode as I2CMode};
use embassy_time::Duration;
use embassy_time::Timer;

pub struct Lcd<'d, T: Instance, M: I2CMode> {
    i2c: I2c<'d, T, M>,
    address: u8,
}

pub enum DisplayControl {
    Off = 0x00,
    CursorBlink = 0x01,
    CursosOn = 0x02,
    DisplayOn = 0x04,
}

#[derive(Copy, Clone)]
pub enum Backlight {
    Off = 0x00,
    On = 0x08,
}

#[repr(u8)]
#[derive(Copy, Clone)]
enum Mode {
    Cmd = 0x00,
    Data = 0x01,
    DisplayControl = 0x08,
    FunctionSet = 0x20,
}

enum Commands {
    Clear = 0x01,
    ReturnHome = 0x02,
    ShiftCursor = 16 | 4,
}

enum BitMode {
    Bit4 = 0x0 << 4,
    Bit8 = 0x1 << 4,
}

// super heavily inspired from https://github.com/KuabeM/lcd-lcm1602-i2c/blob/master/src/lib.rs
// but without delay and i2c interfaces
impl<'d, T: Instance, M: I2CMode> Lcd<'d, T, M> {
    pub async fn try_new(mut i2c: I2c<'d, T, M>, address: u8) -> Result<Self, Error> {
        // Init
        Timer::after(Duration::from_millis(80)).await;

        // Depending on the previously received frame and the LCD's state (mode),
        // we'll need to send the frame up to 3 times.
        // https://badboi.dev/rust,/microcontrollers/2020/11/09/i2c-hello-world.html
        let mode_8bit = Mode::FunctionSet as u8 | BitMode::Bit8 as u8;
        i2c.blocking_write(
            address,
            &[mode_8bit | DisplayControl::DisplayOn as u8 | Backlight::On as u8],
        )?;
        Timer::after(Duration::from_millis(1)).await;
        i2c.blocking_write(address, &[DisplayControl::Off as u8 | Backlight::On as u8])?;
        Timer::after(Duration::from_millis(5)).await;

        i2c.blocking_write(
            address,
            &[mode_8bit | DisplayControl::DisplayOn as u8 | Backlight::On as u8],
        )?;
        Timer::after(Duration::from_millis(1)).await;
        i2c.blocking_write(address, &[DisplayControl::Off as u8 | Backlight::On as u8])?;
        Timer::after(Duration::from_millis(5)).await;

        i2c.blocking_write(
            address,
            &[mode_8bit | DisplayControl::DisplayOn as u8 | Backlight::On as u8],
        )?;
        Timer::after(Duration::from_millis(1)).await;
        i2c.blocking_write(address, &[DisplayControl::Off as u8 | Backlight::On as u8])?;
        Timer::after(Duration::from_millis(5)).await;

        i2c.blocking_write(
            address,
            &[mode_8bit | DisplayControl::DisplayOn as u8 | Backlight::On as u8],
        )?;

        // 4 bit mode
        let mode_4bit = Mode::FunctionSet as u8 | BitMode::Bit4 as u8;
        i2c.blocking_write(
            address,
            &[mode_4bit | DisplayControl::DisplayOn as u8 | Backlight::On as u8],
        )?;
        Timer::after(Duration::from_millis(1)).await;
        i2c.blocking_write(address, &[DisplayControl::Off as u8 | Backlight::On as u8])?;
        Timer::after(Duration::from_millis(5)).await;

        Ok(Self { i2c, address })
    }

    pub async fn hello(&mut self) -> Result<(), Error> {
        for c in "Hello world!".chars() {
            let high_bits: u8 = c as u8 & 0xf0;
            let low_bits: u8 = ((c as u8) << 4) & 0xf0;
            self.i2c.blocking_write(
                self.address,
                &[high_bits | DisplayControl::DisplayOn as u8 | Backlight::On as u8],
            )?;
            Timer::after(Duration::from_millis(1)).await;
            self.i2c.blocking_write(
                self.address,
                &[DisplayControl::Off as u8 | Backlight::On as u8],
            )?;
            self.i2c.blocking_write(
                self.address,
                &[low_bits | DisplayControl::DisplayOn as u8 | Backlight::On as u8],
            )?;
            Timer::after(Duration::from_millis(1)).await;
            self.i2c.blocking_write(
                self.address,
                &[DisplayControl::Off as u8 | Backlight::On as u8],
            )?;
            Timer::after(Duration::from_millis(5)).await;
        }

        Ok(())
    }
}
