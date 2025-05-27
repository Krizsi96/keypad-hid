#![no_std]
#![no_main]

mod keypad;
use core::sync::atomic::{AtomicBool, Ordering};
use defmt::{info, warn};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_stm32::time::Hertz;
use embassy_stm32::usb::Driver;
use embassy_stm32::{Config, bind_interrupts, init, peripherals, usb};
use embassy_time::Timer;
use embassy_usb::class::hid::{HidReaderWriter, ReportId, RequestHandler, State};
use embassy_usb::control::OutResponse;
use embassy_usb::{Builder, Handler};
use usbd_hid::descriptor::{KeyboardReport, SerializedDescriptor};
use {defmt_rtt as _, panic_probe as _};

use crate::keypad::Keypad4x4;

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

    // Create embassy-usb config
    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("HID keyboard example");
    config.serial_number = Some("12345678");

    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut msos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut request_handler = MyRequestHandler {};
    let mut device_handler = MyDeviceHandler::new();

    let mut state = State::new();

    let mut builder = Builder::new(
        usb_driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut msos_descriptor,
        &mut control_buf,
    );

    builder.handler(&mut device_handler);

    // Create classes on the builder.
    let config = embassy_usb::class::hid::Config {
        report_descriptor: KeyboardReport::desc(),
        request_handler: None,
        poll_ms: 60,
        max_packet_size: 8,
    };

    let hid = HidReaderWriter::<_, 1, 8>::new(&mut builder, &mut state, config);

    // Build the builder
    let mut usb = builder.build();

    // Run the USB device.
    let usb_fut = usb.run();

    let (reader, mut writer) = hid.split();

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
            match writer.write_serialize(&report).await {
                Ok(()) => {}
                Err(e) => warn!("Failed to send report: {:?}", e),
            };

            Timer::after_millis(100).await;
        }
    };

    let out_fut = async {
        reader.run(false, &mut request_handler).await;
    };

    join(usb_fut, join(in_fut, out_fut)).await;
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

    check_key!(key_1, 0x1E); // '1'
    check_key!(key_2, 0x1F); // '2'
    check_key!(key_3, 0x20); // '3'
    check_key!(key_a, 0x04); // 'A'
    check_key!(key_4, 0x21); // '4'
    check_key!(key_5, 0x22); // '5'
    check_key!(key_6, 0x23); // '6'
    check_key!(key_b, 0x05); // 'B'
    check_key!(key_7, 0x24); // '7'
    check_key!(key_8, 0x25); // '8'
    check_key!(key_9, 0x26); // '9'
    check_key!(key_c, 0x06); // 'C'
    check_key!(key_star, 0x55); // '*'
    check_key!(key_0, 0x27); // '0'
    check_key!(key_pound, 0x2D); // '-'
    check_key!(key_d, 0x07); // 'D'

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

struct MyRequestHandler {}

impl RequestHandler for MyRequestHandler {
    fn get_report(&mut self, id: ReportId, _buf: &mut [u8]) -> Option<usize> {
        info!("Get report for {:?}", id);
        None
    }

    fn set_report(&mut self, id: ReportId, data: &[u8]) -> OutResponse {
        info!("Set report for {:?}: {=[u8]}", id, data);
        OutResponse::Accepted
    }

    fn get_idle_ms(&mut self, id: Option<ReportId>) -> Option<u32> {
        info!("Get idle rate for {:?}", id);
        None
    }

    fn set_idle_ms(&mut self, id: Option<ReportId>, duration_ms: u32) {
        info!("Set idle rate for {:?} to {:?}", id, duration_ms);
    }
}

struct MyDeviceHandler {
    configured: AtomicBool,
}

impl MyDeviceHandler {
    fn new() -> Self {
        Self {
            configured: AtomicBool::new(false),
        }
    }
}

impl Handler for MyDeviceHandler {
    fn enabled(&mut self, _enabled: bool) {
        self.configured.store(false, Ordering::Relaxed);
        if _enabled {
            info!("Device enabled");
        } else {
            info!("Device disabled");
        }
    }

    fn reset(&mut self) {
        self.configured.store(false, Ordering::Relaxed);
        info!("Bus reset, the Vbus current limit is 100mA");
    }

    fn addressed(&mut self, _addr: u8) {
        self.configured.store(false, Ordering::Relaxed);
        info!("USB address set to: {}", _addr);
    }

    fn configured(&mut self, _configured: bool) {
        self.configured.store(_configured, Ordering::Relaxed);
        if _configured {
            info!(
                "Device configured, it may now draw up to the configured current limit from Vbus."
            );
        } else {
            info!("Device is no longer configured, the Vbus current limit is 100mA.");
        }
    }
}
