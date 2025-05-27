#![no_std]
#![no_main]

mod keypad;

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_time::{Timer};
use {defmt_rtt as _, panic_probe as _};

use crate::keypad::Keypad4x4;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Start main");
    let p = embassy_stm32::init(Default::default());

    let rows = [
        Input::new(p.PA0, Pull::Down),
        Input::new(p.PA1, Pull::Down),
        Input::new(p.PA2, Pull::Down),
        Input::new(p.PA3, Pull::Down),
    ];

    let columns = [
        Output::new(p.PA4, Level::Low, Speed::Low),
        Output::new(p.PA5, Level::Low, Speed::Low),
        Output::new(p.PC4, Level::Low, Speed::Low),
        Output::new(p.PA7, Level::Low, Speed::Low),
    ];

    info!("Create keypad");
    let keypad = Keypad4x4::new(rows, columns);

    spawner.spawn(monitor_keypad(keypad)).unwrap();
}

#[embassy_executor::task]
async fn monitor_keypad(mut keypad: Keypad4x4<Input<'static>, Output<'static>>) {
    loop {
        let key_1: bool = keypad.key_1().into();
        let key_2: bool = keypad.key_2().into();
        let key_3: bool = keypad.key_3().into();
        let key_a: bool = keypad.key_a().into();
        let key_4: bool = keypad.key_4().into();
        let key_5: bool = keypad.key_5().into();
        let key_6: bool = keypad.key_6().into();
        let key_b: bool = keypad.key_b().into();
        let key_7: bool = keypad.key_7().into();
        let key_8: bool = keypad.key_8().into();
        let key_9: bool = keypad.key_9().into();
        let key_c: bool = keypad.key_c().into();
        let key_star: bool = keypad.key_star().into();
        let key_0: bool = keypad.key_0().into();
        let key_pound: bool = keypad.key_pound().into();
        let key_d: bool = keypad.key_d().into();
        info!(
            "\nkey A: {}\nkey B: {}\nkey C: {}\nkey D: {}\nkey *: {}\nkey #: {}\nkey 0: {}\nkey 1: {}\nkey 2: {}\nkey 3: {}\nkey 4: {}\nkey 5: {}\nkey 6: {}\nkey 7: {}\nkey 8: {}\nkey 9: {}",
            key_a,
            key_b,
            key_c,
            key_d,
            key_star,
            key_pound,
            key_0,
            key_1,
            key_2,
            key_3,
            key_4,
            key_5,
            key_6,
            key_7,
            key_8,
            key_9,
        );
        Timer::after_millis(100).await;
    }
}
