use embassy_stm32::Config;
use embassy_stm32::rcc::{
    AHBPrescaler, APBPrescaler, Hse, HseMode, Pll, PllMul, PllPDiv, PllPreDiv, PllQDiv, PllSource,
    Sysclk, mux,
};
use embassy_stm32::time::Hertz;

pub trait UsbConfiguration {
    fn usb_configuration() -> Config;
}

impl UsbConfiguration for Config {
    fn usb_configuration() -> Config {
        let mut config = Config::default();

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

        config
    }
}

pub struct UsbDriverConfig {
    pub ep_out_buffer: [u8; 256],
    pub usb_config: embassy_stm32::usb::Config,
}

impl UsbDriverConfig {
    pub fn new() -> Self {
        let mut config = embassy_stm32::usb::Config::default();
        config.vbus_detection = false;

        Self {
            ep_out_buffer: [0u8; 256],
            usb_config: config,
        }
    }
}
