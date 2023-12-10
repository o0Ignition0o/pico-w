#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), no_main)]
#![feature(type_alias_impl_trait)]

use defmt::Format;
#[cfg(feature = "defmt")]
use defmt_rtt as _;

use panic_probe as _;

use embassy_executor::Executor;
use embassy_time::{Duration, Timer};

use embassy_rp::{
    gpio::{Input, Level, Output, Pin, Pull},
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
    // core0_executor.run(|spawner| spawner.must_spawn(blinky(peripherals)))
    core0_executor.run(|spawner| spawner.must_spawn(dpad_scan(peripherals)))
}

#[cfg(feature = "rp2040")]
#[embassy_executor::task()]
async fn print() {
    loop {
        // info!("Printing on Core 1 every 2 secs...");
        Timer::after(Duration::from_secs(2)).await;
    }
}

#[embassy_executor::task()]
async fn dpad_scan(peripherals: Peripherals) {
    let mut key_pad = KeyPad::new(
        peripherals.PIN_13,
        peripherals.PIN_12,
        peripherals.PIN_11,
        peripherals.PIN_10,
        peripherals.PIN_18,
        peripherals.PIN_19,
        peripherals.PIN_20,
        peripherals.PIN_21,
    );

    loop {
        if let Some(key) = key_pad.pressed_key() {
            info!("Key pressed: {}", key);
        }
    }
}

struct KeyPad<'k, C1, C2, C3, C4, R1, R2, R3, R4>
where
    C1: Pin,
    C2: Pin,
    C3: Pin,
    C4: Pin,
    R1: Pin,
    R2: Pin,
    R3: Pin,
    R4: Pin,
{
    columns: (Input<'k, C1>, Input<'k, C2>, Input<'k, C3>, Input<'k, C4>),
    rows: (
        Output<'k, R1>,
        Output<'k, R2>,
        Output<'k, R3>,
        Output<'k, R4>,
    ),
}

impl<'k, C1, C2, C3, C4, R1, R2, R3, R4> KeyPad<'k, C1, C2, C3, C4, R1, R2, R3, R4>
where
    C1: Pin,
    C2: Pin,
    C3: Pin,
    C4: Pin,
    R1: Pin,
    R2: Pin,
    R3: Pin,
    R4: Pin,
{
    pub fn new(c1: C1, c2: C2, c3: C3, c4: C4, r1: R1, r2: R2, r3: R3, r4: R4) -> Self {
        Self {
            columns: (
                Input::new(c1, Pull::Down),
                Input::new(c2, Pull::Down),
                Input::new(c3, Pull::Down),
                Input::new(c4, Pull::Down),
            ),
            rows: (
                Output::new(r1, Level::Low),
                Output::new(r2, Level::Low),
                Output::new(r3, Level::Low),
                Output::new(r4, Level::Low),
            ),
        }
    }

    pub fn pressed_key(&mut self) -> Option<Key> {
        let all_buttons = [
            [Key::One, Key::Two, Key::Three, Key::A],
            [Key::Four, Key::Five, Key::Six, Key::B],
            [Key::Seven, Key::Eight, Key::Nine, Key::C],
            [Key::Wildcard, Key::Zero, Key::Pound, Key::D],
        ];

        let buttons = self.pressed_button();

        for (row, column) in buttons.iter().enumerate() {
            for (col, is_pressed) in column.iter().enumerate() {
                if *is_pressed {
                    return Some(all_buttons[row][col].clone());
                }
            }
        }

        None
    }

    fn pressed_button(&mut self) -> [[bool; 4]; 4] {
        macro_rules! for_row {
            ($row:expr) => {{
                $row.set_high();
                let res = [
                    self.columns.0.is_high(),
                    self.columns.1.is_high(),
                    self.columns.2.is_high(),
                    self.columns.3.is_high(),
                ];
                $row.set_low();
                res
            }};
        }

        [
            for_row!(self.rows.0),
            for_row!(self.rows.1),
            for_row!(self.rows.2),
            for_row!(self.rows.3),
        ]
    }
}

#[derive(Clone)]
enum Key {
    One,
    Two,
    Three,
    A,
    Four,
    Five,
    Six,
    B,
    Seven,
    Eight,
    Nine,
    C,
    Wildcard,
    Zero,
    Pound,
    D,
}

impl Format for Key {
    fn format(&self, fmt: defmt::Formatter) {
        let c = match self {
            Self::A => 'A',
            Self::B => 'B',
            Self::C => 'C',
            Self::D => 'D',
            Self::One => '1',
            Self::Two => '2',
            Self::Three => '3',
            Self::Four => '4',
            Self::Five => '5',
            Self::Six => '6',
            Self::Seven => '7',
            Self::Eight => '8',
            Self::Nine => '9',
            Self::Zero => '0',
            Self::Wildcard => '*',
            Self::Pound => '#',
        };
        defmt::write!(fmt, "{}", c);
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
