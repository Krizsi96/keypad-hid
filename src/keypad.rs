use embedded_hal::digital::{InputPin, OutputPin, PinState};

pub struct Keypad4x4<T, U>
where
    T: InputPin,
    U: OutputPin,
{
    rows: [T; 4],
    columns: [U; 4],
}

impl<T, U> Keypad4x4<T, U>
where
    T: InputPin,
    U: OutputPin,
{
    pub fn new(mut rows: [T; 4], mut columns: [U; 4]) -> Keypad4x4<T, U> {
        if rows.iter_mut().any(|row| row.is_high().unwrap()) {
            panic!("Input pins should be pulled low");
        }
        Self { rows, columns }
    }

    pub fn key_0(&mut self) -> PinState {
        self.check_key_state(Key::Num0)
    }

    pub fn key_1(&mut self) -> PinState {
        self.check_key_state(Key::Num1)
    }

    pub fn key_2(&mut self) -> PinState {
        self.check_key_state(Key::Num2)
    }

    pub fn key_3(&mut self) -> PinState {
        self.check_key_state(Key::Num3)
    }

    pub fn key_4(&mut self) -> PinState {
        self.check_key_state(Key::Num4)
    }

    pub fn key_5(&mut self) -> PinState {
        self.check_key_state(Key::Num5)
    }

    pub fn key_6(&mut self) -> PinState {
        self.check_key_state(Key::Num6)
    }

    pub fn key_7(&mut self) -> PinState {
        self.check_key_state(Key::Num7)
    }

    pub fn key_8(&mut self) -> PinState {
        self.check_key_state(Key::Num8)
    }

    pub fn key_9(&mut self) -> PinState {
        self.check_key_state(Key::Num9)
    }

    pub fn key_a(&mut self) -> PinState {
        self.check_key_state(Key::A)
    }

    pub fn key_b(&mut self) -> PinState {
        self.check_key_state(Key::B)
    }

    pub fn key_c(&mut self) -> PinState {
        self.check_key_state(Key::C)
    }

    pub fn key_d(&mut self) -> PinState {
        self.check_key_state(Key::D)
    }

    pub fn key_star(&mut self) -> PinState {
        self.check_key_state(Key::Star)
    }

    pub fn key_pound(&mut self) -> PinState {
        self.check_key_state(Key::Pound)
    }

    fn check_key_state(&mut self, key: Key) -> PinState {
        let key_position = key.get_indexes();
        self.set_outputs(PinState::Low);
        self.columns
            .get_mut(key_position.column_index)
            .expect("Invalid column index")
            .set_high()
            .unwrap();
        let is_pressed = self
            .rows
            .get_mut(key_position.row_index)
            .expect("Invalid row index")
            .is_high()
            .unwrap();
        self.set_outputs(PinState::High);
        is_pressed.into()
    }

    fn set_outputs(&mut self, state: PinState) {
        self.columns.iter_mut().for_each(|output| match state {
            PinState::Low => output.set_low().unwrap(),
            PinState::High => output.set_high().unwrap(),
        });
    }
}

struct KeyPosition {
    row_index: usize,
    column_index: usize,
}

enum Key {
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    A,
    B,
    C,
    D,
    Star,
    Pound,
}

impl Key {
    fn get_indexes(self) -> KeyPosition {
        match self {
            Key::Num1 => KeyPosition {
                row_index: 0,
                column_index: 0,
            },
            Key::Num2 => KeyPosition {
                row_index: 0,
                column_index: 1,
            },
            Key::Num3 => KeyPosition {
                row_index: 0,
                column_index: 2,
            },
            Key::A => KeyPosition {
                row_index: 0,
                column_index: 3,
            },
            Key::Num4 => KeyPosition {
                row_index: 1,
                column_index: 0,
            },
            Key::Num5 => KeyPosition {
                row_index: 1,
                column_index: 1,
            },
            Key::Num6 => KeyPosition {
                row_index: 1,
                column_index: 2,
            },
            Key::B => KeyPosition {
                row_index: 1,
                column_index: 3,
            },
            Key::Num7 => KeyPosition {
                row_index: 2,
                column_index: 0,
            },
            Key::Num8 => KeyPosition {
                row_index: 2,
                column_index: 1,
            },
            Key::Num9 => KeyPosition {
                row_index: 2,
                column_index: 2,
            },
            Key::C => KeyPosition {
                row_index: 2,
                column_index: 3,
            },
            Key::Star => KeyPosition {
                row_index: 3,
                column_index: 0,
            },
            Key::Num0 => KeyPosition {
                row_index: 3,
                column_index: 1,
            },
            Key::Pound => KeyPosition {
                row_index: 3,
                column_index: 2,
            },
            Key::D => KeyPosition {
                row_index: 3,
                column_index: 3,
            },
        }
    }
}
