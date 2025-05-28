#![no_std]
#![no_main]

mod keypad;
mod usb_keyboard;

use defmt::{info, warn};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_stm32::time::Hertz;
use embassy_stm32::usb::Driver;
use embassy_stm32::{Config, bind_interrupts, init, peripherals, usb};
use embassy_time::Timer;
use usbd_hid::descriptor::{KeyboardReport, KeyboardUsage};
use {defmt_rtt as _, panic_probe as _};

use crate::keypad::Keypad4x4;
use crate::usb_keyboard::UsbKeyboard;

bind_interrupts!(struct Irqs {
    OTG_FS => usb::InterruptHandler<peripherals::USB_OTG_FS>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Start main");
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: Hertz(8_000_000),
            mode: HseMode::Bypass,
        });
        config.rcc.pll_src = PllSource::HSE;
        config.rcc.pll = Some(Pll {
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL168,
            divp: Some(PllPDiv::DIV2), // 8 Mhz / 4 * 168 / 2 = 168Mhz
            divq: Some(PllQDiv::DIV7), // 8 Mhz / 4 * 168 / 7 = 48 Mhz
            divr: None,
        });
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV4;
        config.rcc.apb2_pre = APBPrescaler::DIV2;
        config.rcc.sys = Sysclk::PLL1_P;
        config.rcc.mux.clk48sel = mux::Clk48sel::PLL1_Q;
    }

    let p = init(config);

    // Create the driver, from HAL
    let mut ep_out_buffer = [0u8; 256];
    let mut config = embassy_stm32::usb::Config::default();
    config.vbus_detection = false;

    let usb_driver = Driver::new_fs(
        p.USB_OTG_FS,
        Irqs,
        p.PA12,
        p.PA11,
        &mut ep_out_buffer,
        config,
    );

    let mut usb_keyboard_config = usb_keyboard::Config::default();
    let mut usb_keyboard = UsbKeyboard::new(&mut usb_keyboard_config, usb_driver);

    info!("Create keypad");
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
    let mut keypad = Keypad4x4::new(rows, columns);

    // Report keystroke
    let in_fut = async {
        loop {
            let keycodes = check_keypad_buttons(&mut keypad);

            let report = KeyboardReport {
                keycodes,
                leds: 0,
                modifier: 0,
                reserved: 0,
            };

            // Send the report
            match usb_keyboard.hid_writer.write_serialize(&report).await {
                Ok(()) => {}
                Err(e) => warn!("Failed to send report: {:?}", e),
            };

            Timer::after_millis(100).await;
        }
    };

    let out_fut = async {
        usb_keyboard.hid_reader.run(false, usb_keyboard.request_handler).await;
    };

    let usb_future = usb_keyboard.usb.run();

    join(usb_future, join(in_fut, out_fut)).await;
}

fn check_keypad_buttons(keypad: &mut Keypad4x4<Input<'static>, Output<'static>>) -> [u8; 6] {
    let mut keycodes: [u8; 6] = [0; 6];
    let mut index = 0;

    macro_rules! check_key {
        ($pressed:expr, $hid_code:expr) => {
            if $pressed && index < 6 {
                keycodes[index] = $hid_code;
                index += 1;
            }
        };
    }

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

    check_key!(key_1, KeyboardUsage::Keyboard1Exclamation as u8);
    check_key!(key_2, KeyboardUsage::Keyboard2At as u8);
    check_key!(key_3, KeyboardUsage::Keyboard3Hash as u8);
    check_key!(key_a, KeyboardUsage::KeyboardAa as u8);
    check_key!(key_4, KeyboardUsage::Keyboard4Dollar as u8);
    check_key!(key_5, KeyboardUsage::Keyboard5Percent as u8);
    check_key!(key_6, KeyboardUsage::Keyboard6Caret as u8);
    check_key!(key_b, KeyboardUsage::KeyboardBb as u8);
    check_key!(key_7, KeyboardUsage::Keyboard7Ampersand as u8);
    check_key!(key_8, KeyboardUsage::Keyboard8Asterisk as u8);
    check_key!(key_9, KeyboardUsage::Keyboard9OpenParens as u8);
    check_key!(key_c, KeyboardUsage::KeyboardCc as u8);
    check_key!(key_star, KeyboardUsage::KeypadMultiply as u8);
    check_key!(key_0, KeyboardUsage::Keyboard0CloseParens as u8);
    check_key!(key_pound, KeyboardUsage::KeyboardDashUnderscore as u8);
    check_key!(key_d, KeyboardUsage::KeyboardDd as u8);

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

    info!("keycodes: {}", keycodes);

    keycodes
}
