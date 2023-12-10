#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), no_main)]
#![feature(type_alias_impl_trait)]

#[cfg(feature = "defmt")]
use defmt_rtt as _;

use panic_probe as _;

use embassy_executor::Executor;
use embassy_time::{Duration, Timer};

use embassy_rp::{
    gpio::{Level, Output},
    multicore::{spawn_core1, Stack},
    Peripheral, Peripherals,
};

use cortex_m_rt::{exception, ExceptionFrame};

use embassy_pico_template::info;

use static_cell::StaticCell;

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

    spawn_core1(
        unsafe { peripherals.CORE1.clone_unchecked() },
        unsafe { &mut CORE1_STACK },
        move || {
            let core1_executor = CORE1_EXECUTOR.init(Executor::new());

            core1_executor.run(|spawner| spawner.must_spawn(print()))
        },
    );

    let core0_executor = CORE0_EXECUTOR.init(Executor::new());
    core0_executor.run(|spawner| spawner.must_spawn(blinky(peripherals)))
}

#[cfg(feature = "rp2040")]
#[embassy_executor::task()]
async fn print() {
    loop {
        info!("Printing on Core 1 every 2 secs...");
        Timer::after(Duration::from_secs(2)).await;
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
