use std::sync::Arc;

use esp_idf_sys as _;
// If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use log::info;

use esp_idf_hal::delay::Delay;
use esp_idf_hal::gpio;
use esp_idf_hal::gpio::*;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;
use esp_idf_hal::spi::{self, SpiDriver, SpiSharedDeviceDriver, SpiSoftCsDeviceDriver};

#[allow(unused)]
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{
        Circle, Line, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, StrokeAlignment, Triangle,
    },
    text::{Alignment, Text},
};

use epd_waveshare::prelude::*;
use epd_waveshare::{buffer_len, epd12in48b_v2 as epd, graphics::VarDisplay};

pub fn main() {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("wakeup: {:?}", esp_idf_hal::reset::WakeupReason::get());

    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    let sclk = pins.gpio13;
    let mosi = pins.gpio14;

    let m1_cs = pins.gpio23;
    let s1_cs = pins.gpio22;
    let m2_cs = pins.gpio16;
    let s2_cs = pins.gpio19;

    let m1s1_dc = pins.gpio25;
    let m2s2_dc = pins.gpio17;

    let m1s1_rst = pins.gpio33;
    let m2s2_rst = pins.gpio5;

    let m1_busy = pins.gpio32;
    let s1_busy = pins.gpio26;
    let m2_busy = pins.gpio18;
    let s2_busy = pins.gpio4;

    let spi_host = SpiDriver::new(
        peripherals.spi3,
        sclk,
        mosi,
        Option::<gpio::AnyIOPin>::None,
        &spi::config::DriverConfig::new(),
    )
    .unwrap();

    let spi_device = SpiSharedDeviceDriver::new(
        spi_host,
        &spi::config::Config::new()
            .baudrate(10.MHz().into())
            .duplex(spi::config::Duplex::Half3Wire),
    )
    .unwrap();

    let spi_device = Arc::new(spi_device);

    type OutputPinDriver = PinDriver<'static, AnyOutputPin, Output>;
    type InputPinDriver = PinDriver<'static, AnyInputPin, Input>;
    type SPIDevice = SpiSoftCsDeviceDriver<
        'static,
        Arc<SpiSharedDeviceDriver<'static, SpiDriver<'static>>>,
        SpiDriver<'static>,
    >;

    let peris = epd::Peripherals::<InputPinDriver, OutputPinDriver, SPIDevice> {
        m1: SpiSoftCsDeviceDriver::new(spi_device.clone(), m1_cs, Level::High).unwrap(),
        s1: SpiSoftCsDeviceDriver::new(spi_device.clone(), s1_cs, Level::High).unwrap(),
        m2: SpiSoftCsDeviceDriver::new(spi_device.clone(), m2_cs, Level::High).unwrap(),
        s2: SpiSoftCsDeviceDriver::new(spi_device.clone(), s2_cs, Level::High).unwrap(),
        m1s1_dc: OutputPinDriver::output(m1s1_dc.downgrade_output()).unwrap(),
        m2s2_dc: OutputPinDriver::output(m2s2_dc.downgrade_output()).unwrap(),
        m1s1_rst: OutputPinDriver::output(m1s1_rst.downgrade_output()).unwrap(),
        m2s2_rst: OutputPinDriver::output(m2s2_rst.downgrade_output()).unwrap(),
        m1_busy: InputPinDriver::input(m1_busy.downgrade_input()).unwrap(),
        s1_busy: InputPinDriver::input(s1_busy.downgrade_input()).unwrap(),
        m2_busy: InputPinDriver::input(m2_busy.downgrade_input()).unwrap(),
        s2_busy: InputPinDriver::input(s2_busy.downgrade_input()).unwrap(),
    };

    let mut epd_driver = epd::EpdDriver::<InputPinDriver, OutputPinDriver, SPIDevice, Delay>::new(
        peris,
        Delay::new_default(),
    );

    info!("reset");
    epd_driver.reset().unwrap();

    info!("init");
    epd_driver
        .init(&epd::Config {
            kwr: true,
            kw_new_to_old: false,
            ..Default::default()
        })
        .unwrap();

    let mut buffer = Vec::new();
    buffer.resize(buffer_len(epd::WIDTH as usize, epd::HEIGHT as usize), 0);
    let mut fb = VarDisplay::<Color>::new(epd::WIDTH, epd::HEIGHT, &mut buffer, false).unwrap();
    fb.clear(Color::White).unwrap();

    info!("write_bank1");
    epd_driver.write_bank1(fb.buffer()).unwrap();

    fb.clear(Color::Black).unwrap();
    let line_style = PrimitiveStyleBuilder::new()
        .stroke_width(1)
        .stroke_color(Color::White)
        .build();
    Line::new(Point::new(0, 0), Point::new(1303, 983))
        .into_styled(line_style)
        .draw(&mut fb)
        .unwrap();
    Line::new(Point::new(0, 983), Point::new(1303, 0))
        .into_styled(line_style)
        .draw(&mut fb)
        .unwrap();

    fn write_text(fb: &mut VarDisplay<'_, Color>, pos: Point, text: &str) {
        let character_style = MonoTextStyle::new(&FONT_10X20, Color::White);
        Text::with_alignment(&text, pos, character_style, Alignment::Left)
            .draw(fb)
            .unwrap();
    }
    write_text(&mut fb, Point::new(20, 20), "S2 S2 S2 S2");
    write_text(&mut fb, Point::new(660, 20), "M2 M2 M2 M2");
    write_text(&mut fb, Point::new(20, 510), "M1 M1 M1 M1");
    write_text(&mut fb, Point::new(660, 510), "S1 S1 S1 S1");

    info!("write_bank2");
    epd_driver.write_bank2(fb.buffer()).unwrap();

    info!("refresh_display");
    epd_driver.refresh_display().unwrap();

    info!("shutdown");
    epd_driver.shutdown().unwrap();

    info!("exiting");
}
