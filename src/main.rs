#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), no_main)]
#![feature(type_alias_impl_trait)]

#[cfg(feature = "defmt")]
use defmt_rtt as _;
use panic_probe as _;

use cortex_m_rt::{exception, ExceptionFrame};
use embassy_executor::Executor;
use embassy_pico_template::info;
use embassy_rp::{
    gpio::{Level, Output},
    multicore::{spawn_core1, Stack},
    Peripheral, Peripherals,
};
use embassy_time::{Duration, Timer};
use static_cell::StaticCell;

use embassy_pico_template::keypad::KeyPad;

#[cfg(feature = "cortex-m")]
/// Stack - Core1 stack = Core 0 stack size.
static CORE0_EXECUTOR: StaticCell<Executor> = StaticCell::new();
#[cfg(feature = "cortex-m")]
static CORE1_EXECUTOR: StaticCell<Executor> = StaticCell::new();
#[cfg(feature = "cortex-m")]
// TODO: Set a stack size for the second core
static mut CORE1_STACK: Stack<{ 30 * 1024 }> = Stack::new();

#[cortex_m_rt::entry]
fn main() -> ! {
    embassy_rp::pac::SIO.spinlock(31).write_value(1);

    let peripherals = embassy_rp::init(Default::default());

    let (i2c, sda, scl) = (peripherals.I2C1, peripherals.PIN_26, peripherals.PIN_27);
    let (one, two, three, four, five, six, seven, eight) = (
        peripherals.PIN_13,
        peripherals.PIN_12,
        peripherals.PIN_11,
        peripherals.PIN_10,
        peripherals.PIN_18,
        peripherals.PIN_19,
        peripherals.PIN_20,
        peripherals.PIN_21,
    );

    spawn_core1(
        unsafe { peripherals.CORE1.clone_unchecked() },
        unsafe { &mut CORE1_STACK },
        move || {
            let core1_executor = CORE1_EXECUTOR.init(Executor::new());

            core1_executor.run(|spawner| spawner.must_spawn(lcd(i2c, sda, scl)))
        },
    );

    let core0_executor = CORE0_EXECUTOR.init(Executor::new());
    // core0_executor.run(|spawner| spawner.must_spawn(blinky(peripherals)))
    core0_executor.run(|spawner| {
        spawner.must_spawn(keypad_scan(one, two, three, four, five, six, seven, eight))
    })
}

#[cfg(feature = "rp2040")]
#[embassy_executor::task()]
async fn print() {
    loop {
        info!("Printing on Core 1 every 2 secs...");
        Timer::after(Duration::from_secs(2)).await;
    }
}

#[cfg(feature = "rp2040")]
#[embassy_executor::task()]
async fn lcd(
    i2c: embassy_rp::peripherals::I2C1,
    sda: embassy_rp::peripherals::PIN_26,
    scl: embassy_rp::peripherals::PIN_27,
) {
    use embassy_pico_template::lcd::Lcd;
    use embassy_rp::{
        bind_interrupts,
        i2c::{Config, I2c, InterruptHandler},
        peripherals::I2C1,
    };

    bind_interrupts!(struct Irqs {
        I2C1_IRQ => InterruptHandler<I2C1>;
    });

    let i2c = I2c::new_async(i2c, scl, sda, Irqs, Config::default());
    const LCD_ADDRESS: u8 = 0x27;

    let mut lcd = Lcd::try_new(i2c, LCD_ADDRESS).await.unwrap();

    lcd.hello().await.unwrap();
    loop {
        // info!("Printing on Core 1 every 2 secs...");
        Timer::after(Duration::from_secs(2)).await;
    }
}

#[embassy_executor::task()]
async fn keypad_scan(
    one: embassy_rp::peripherals::PIN_13,
    two: embassy_rp::peripherals::PIN_12,
    three: embassy_rp::peripherals::PIN_11,
    four: embassy_rp::peripherals::PIN_10,
    five: embassy_rp::peripherals::PIN_18,
    six: embassy_rp::peripherals::PIN_19,
    seven: embassy_rp::peripherals::PIN_20,
    eight: embassy_rp::peripherals::PIN_21,
) {
    let mut key_pad = KeyPad::new(one, two, three, four, five, six, seven, eight);

    loop {
        if let Some(key) = key_pad.pressed_key() {
            info!("Key pressed: {}", key);
        }
    }
}

#[embassy_executor::task()]
async fn blinky(peripherals: Peripherals) {
    let mut blue_led = Output::new(peripherals.PIN_14, Level::Low);
    let mut red_led = Output::new(peripherals.PIN_17, Level::Low);
    let mut onboard_led = Output::new(peripherals.PIN_25, Level::Low);

    loop {
        info!("blue!");
        blue_led.set_high();
        red_led.set_low();
        onboard_led.set_low();
        Timer::after(Duration::from_secs(1)).await;

        info!("red!");
        blue_led.set_low();
        red_led.set_high();
        onboard_led.set_low();
        Timer::after(Duration::from_secs(1)).await;
        info!("onboard!");
        blue_led.set_low();
        red_led.set_low();
        onboard_led.set_high();
        Timer::after(Duration::from_secs(1)).await;
    }
}

#[cfg(feature = "cortex-m")]
#[exception]
unsafe fn HardFault(ef: &ExceptionFrame) -> ! {
    use embassy_pico_template::error;

    #[cfg(feature = "defmt")]
    error!("HardFault: {:#?}", defmt::Debug2Format(ef));

    #[cfg(not(feature = "defmt"))]
    error!("HardFault: {:#?}", ef);

    loop {}
}

#[cfg(not(feature = "cortex-m"))]
fn main() {}
