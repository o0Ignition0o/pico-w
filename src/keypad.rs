use defmt::Format;
use embassy_rp::gpio::{Input, Level, Output, Pin, Pull};

pub struct KeyPad<'k, C1, C2, C3, C4, R1, R2, R3, R4>
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
    last_pressed_key: Option<Key>,
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
            last_pressed_key: None,
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

        let mut last_pressed_key = None;
        for (row, column) in buttons.iter().enumerate() {
            for (col, is_pressed) in column.iter().enumerate() {
                if *is_pressed {
                    last_pressed_key = Some(all_buttons[row][col]);
                }
            }
        }

        if last_pressed_key != self.last_pressed_key {
            self.last_pressed_key = last_pressed_key;
            self.last_pressed_key
        } else {
            None
        }
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Key {
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
