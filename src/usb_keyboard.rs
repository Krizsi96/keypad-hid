use core::sync::atomic::{AtomicBool, Ordering};
use defmt::info;
use embassy_stm32::peripherals::USB_OTG_FS;
use embassy_stm32::usb::Driver;
use embassy_usb::class::hid::{
    HidReader, HidReaderWriter, HidWriter, ReportId, RequestHandler, State,
};
use embassy_usb::control::OutResponse;
use embassy_usb::{Builder, Handler, UsbDevice};
use usbd_hid::descriptor::{KeyboardReport, SerializedDescriptor};

pub struct UsbKeyboard<'a> {
    pub usb: UsbDevice<'a, Driver<'a, USB_OTG_FS>>,
    pub hid_reader: HidReader<'a, Driver<'a, USB_OTG_FS>, 1>,
    pub hid_writer: HidWriter<'a, Driver<'a, USB_OTG_FS>, 8>,
    pub request_handler: &'a mut UsbKeyboardRequestHandler,
}

impl<'a> UsbKeyboard<'a> {
    pub fn new(config: &'a mut Config<'a>, driver: Driver<'a, USB_OTG_FS>) -> UsbKeyboard<'a> {
        let mut builder = Builder::new(
            driver,
            config.embassy_config,
            &mut config.config_descriptor,
            &mut config.bos_descriptor,
            &mut config.msos_descriptor,
            &mut config.control_buf,
        );

        builder.handler(&mut config.device_handler);

        let class_config = embassy_usb::class::hid::Config {
            report_descriptor: KeyboardReport::desc(),
            request_handler: None,
            poll_ms: 60,
            max_packet_size: 8,
        };

        let (hid_reader, hid_writer) =
            HidReaderWriter::<_, 1, 8>::new(&mut builder, &mut config.hid_state, class_config)
                .split();

        Self {
            usb: builder.build(),
            hid_reader,
            hid_writer,
            request_handler: &mut config.request_handler,
        }
    }
}

pub struct Config<'a> {
    embassy_config: embassy_usb::Config<'a>,
    config_descriptor: [u8; 256],
    bos_descriptor: [u8; 256],
    msos_descriptor: [u8; 256],
    control_buf: [u8; 64],
    request_handler: UsbKeyboardRequestHandler,
    device_handler: UsbKeyboardDeviceHandler,
    hid_state: State<'a>,
}

impl Config<'_> {
    pub const fn new() -> Self {
        // Create embassy-usb config
        let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
        config.manufacturer = Some("Embassy");
        config.product = Some("HID keyboard example");
        config.serial_number = Some("12345678");

        Self {
            embassy_config: config,
            config_descriptor: [0; 256],
            bos_descriptor: [0; 256],
            msos_descriptor: [0; 256],
            control_buf: [0; 64],
            request_handler: UsbKeyboardRequestHandler::new(),
            device_handler: UsbKeyboardDeviceHandler::new(),
            hid_state: State::new(),
        }
    }
}

pub struct UsbKeyboardRequestHandler {}

impl UsbKeyboardRequestHandler {
    const fn new() -> Self {
        Self {}
    }
}

impl RequestHandler for UsbKeyboardRequestHandler {
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

struct UsbKeyboardDeviceHandler {
    configured: AtomicBool,
}

impl UsbKeyboardDeviceHandler {
    const fn new() -> Self {
        Self {
            configured: AtomicBool::new(false),
        }
    }
}

impl Handler for UsbKeyboardDeviceHandler {
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
