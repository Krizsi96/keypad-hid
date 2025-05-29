#![no_std]
#![no_main]

mod board_pinout;
mod keypad;
mod stm32_configuration;
mod usb_keyboard;

use crate::board_pinout::Board;
use crate::keypad::Keypad4x4;
use crate::stm32_configuration::UsbDriverConfig;
use crate::usb_keyboard::{UsbKeyboard, UsbKeyboardRequestHandler};
use defmt::{debug, info, warn};
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Input, Output};
use embassy_stm32::peripherals::USB_OTG_FS;
use embassy_stm32::usb::Driver;
use embassy_stm32::{Config, init};
use embassy_time::Timer;
use embassy_usb::UsbDevice;
use embassy_usb::class::hid::{HidReader, HidWriter};
use static_cell::StaticCell;
use stm32_configuration::UsbConfiguration;
use usbd_hid::descriptor::{KeyboardReport, KeyboardUsage};
use {defmt_rtt as _, panic_probe as _};

static USB_DRIVER_CONFIG: StaticCell<UsbDriverConfig> = StaticCell::new();
static USB_KEYBOARD_CONFIG: StaticCell<usb_keyboard::Config> = StaticCell::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Start main");
    let peripherals = init(Config::usb_configuration());
    let board = Board::new(peripherals);

    info!("Create USB Driver");
    let usb_driver_config = USB_DRIVER_CONFIG.init(UsbDriverConfig::new());
    let usb_driver = Driver::new_fs(
        board.usb_peripheral,
        board.usb_interrupt,
        board.usb_d_plus,
        board.usb_d_minus,
        &mut usb_driver_config.ep_out_buffer,
        usb_driver_config.usb_config,
    );

    info!("Create USB keyboard device");
    let usb_keyboard_config = USB_KEYBOARD_CONFIG.init(usb_keyboard::Config::new());
    let usb_keyboard = UsbKeyboard::new(usb_keyboard_config, usb_driver);

    info!("Create keypad I/O");
    let keypad = Keypad4x4::new(board.keypad_rows, board.keypad_columns);

    spawner.spawn(usb_run(usb_keyboard.usb)).unwrap();
    spawner
        .spawn(hid_read(
            usb_keyboard.hid_reader,
            usb_keyboard.request_handler,
        ))
        .unwrap();
    spawner
        .spawn(report_keystrokes(
            usb_keyboard.hid_writer,
            keypad,
            board.keypad_interrupt,
        ))
        .unwrap();
}

#[embassy_executor::task]
async fn usb_run(mut usb: UsbDevice<'static, Driver<'static, USB_OTG_FS>>) {
    info!("Start 'USB Run' task");
    usb.run().await;
}

#[embassy_executor::task]
async fn hid_read(
    hid_reader: HidReader<'static, Driver<'static, USB_OTG_FS>, 1>,
    request_handler: &'static mut UsbKeyboardRequestHandler,
) {
    info!("Start 'HID Read' task");
    hid_reader.run(false, request_handler).await;
}

#[embassy_executor::task]
async fn report_keystrokes(
    mut hid_writer: HidWriter<'static, Driver<'static, USB_OTG_FS>, 8>,
    mut keypad: Keypad4x4<Input<'static>, Output<'static>>,
    mut keypad_interrupt: ExtiInput<'static>,
) {
    info!("Start 'Report Key Strokes' task");
    loop {
        keypad_interrupt.wait_for_high().await;
        let keycodes = check_keypad_buttons(&mut keypad);

        // Send the report
        let report = KeyboardReport {
            keycodes,
            leds: 0,
            modifier: 0,
            reserved: 0,
        };
        match hid_writer.write_serialize(&report).await {
            Ok(()) => {}
            Err(e) => warn!("Failed to send report: {:?}", e),
        };

        // Send an empty report to avoid "sticky" keys
        let report = KeyboardReport {
            keycodes: [0, 0, 0, 0, 0, 0],
            leds: 0,
            modifier: 0,
            reserved: 0,
        };
        match hid_writer.write_serialize(&report).await {
            Ok(()) => {}
            Err(e) => warn!("Failed to send report: {:?}", e),
        };

        Timer::after_millis(150).await;
    }
}

fn check_keypad_buttons(keypad: &mut Keypad4x4<Input<'static>, Output<'static>>) -> [u8; 6] {
    let keys = [
        (
            keypad.key_1().into(),
            KeyboardUsage::Keyboard1Exclamation as u8,
        ),
        (keypad.key_2().into(), KeyboardUsage::Keyboard2At as u8),
        (keypad.key_3().into(), KeyboardUsage::Keyboard3Hash as u8),
        (keypad.key_a().into(), KeyboardUsage::KeyboardAa as u8),
        (keypad.key_4().into(), KeyboardUsage::Keyboard4Dollar as u8),
        (keypad.key_5().into(), KeyboardUsage::Keyboard5Percent as u8),
        (keypad.key_6().into(), KeyboardUsage::Keyboard6Caret as u8),
        (keypad.key_b().into(), KeyboardUsage::KeyboardBb as u8),
        (
            keypad.key_7().into(),
            KeyboardUsage::Keyboard7Ampersand as u8,
        ),
        (
            keypad.key_8().into(),
            KeyboardUsage::Keyboard8Asterisk as u8,
        ),
        (
            keypad.key_9().into(),
            KeyboardUsage::Keyboard9OpenParens as u8,
        ),
        (keypad.key_c().into(), KeyboardUsage::KeyboardCc as u8),
        (
            keypad.key_star().into(),
            KeyboardUsage::KeypadMultiply as u8,
        ),
        (
            keypad.key_0().into(),
            KeyboardUsage::Keyboard0CloseParens as u8,
        ),
        (
            keypad.key_pound().into(),
            KeyboardUsage::KeyboardDashUnderscore as u8,
        ),
        (keypad.key_d().into(), KeyboardUsage::KeyboardDd as u8),
    ];

    // Fill keycodes with up to 6 pressed keys
    let mut keycodes: [u8; 6] = [0; 6];
    for (i, (_, code)) in keys
        .iter()
        .filter(|(pressed, _)| *pressed)
        .take(6)
        .enumerate()
    {
        keycodes[i] = *code;
    }

    debug!(
        "\nkey 1: {}\nkey 2: {}\nkey 3: {}\nkey A: {}\nkey 4: {}\nkey 5: {}\nkey 6: {}\nkey B: {}\nkey 7: {}\nkey 8: {}\nkey 9: {}\nkey C: {}\nkey *: {}\nkey 0: {}\nkey #: {}\nkey D: {}",
        keys[0].0,  // key 1
        keys[1].0,  // key 2
        keys[2].0,  // key 3
        keys[3].0,  // key A
        keys[4].0,  // key 4
        keys[5].0,  // key 5
        keys[6].0,  // key 6
        keys[7].0,  // key B
        keys[8].0,  // key 7
        keys[9].0,  // key 8
        keys[10].0, // key 9
        keys[11].0, // key C
        keys[12].0, // key *
        keys[13].0, // key 0
        keys[14].0, // key #
        keys[15].0, // key D
    );
    debug!("keycodes: {}", keycodes);

    keycodes
}
