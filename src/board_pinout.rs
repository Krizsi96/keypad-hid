use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_stm32::peripherals::{PA11, PA12, USB_OTG_FS};
use embassy_stm32::{Peripherals, bind_interrupts, usb};

bind_interrupts!(pub struct Irqs {
    OTG_FS => usb::InterruptHandler<USB_OTG_FS>;
});

pub struct Board {
    pub usb_peripheral: USB_OTG_FS,
    pub usb_interrupt: Irqs,
    pub usb_d_plus: PA12,
    pub usb_d_minus: PA11,
    pub keypad_rows: [Input<'static>; 4],
    pub keypad_columns: [Output<'static>; 4],
    pub keypad_interrupt: ExtiInput<'static>,
}

impl Board {
    pub fn new(peripherals: Peripherals) -> Self {
        Self {
            usb_peripheral: peripherals.USB_OTG_FS,
            usb_interrupt: Irqs,
            usb_d_plus: peripherals.PA12,
            usb_d_minus: peripherals.PA11,
            keypad_rows: [
                Input::new(peripherals.PA0, Pull::Down),
                Input::new(peripherals.PA1, Pull::Down),
                Input::new(peripherals.PA2, Pull::Down),
                Input::new(peripherals.PA3, Pull::Down),
            ],
            keypad_columns: [
                Output::new(peripherals.PA4, Level::High, Speed::Low),
                Output::new(peripherals.PA5, Level::High, Speed::Low),
                Output::new(peripherals.PC4, Level::High, Speed::Low),
                Output::new(peripherals.PA7, Level::High, Speed::Low),
            ],
            keypad_interrupt: ExtiInput::new(peripherals.PB1, peripherals.EXTI1, Pull::Down),
        }
    }
}
