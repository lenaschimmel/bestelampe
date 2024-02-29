use std::error::Error;
use esp_idf_hal::delay::Delay;
use esp_idf_hal::gpio::*;
use esp_idf_hal::spi::*;
use esp_idf_hal::units::FromValueType;

use esp_idf_hal::peripherals::Peripherals;
use embedded_hal::spi::MODE_3;
use esp_idf_hal::units::FromValueType::*;

use embedded_graphics::{
    pixelcolor::Gray4,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};

fn main() -> Result<(), Box<dyn Error>> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    let busy = PinDriver::input(peripherals.pins.gpio27);
    let rst = PinDriver::output(peripherals.pins.gpio2); // MAIN_PWR_PINM

    let spi = peripherals.spi2;
    let sclk = peripherals.pins.gpio14;
    let sdo = peripherals.pins.gpio12; // Mosi == 12, not sure if sdo = mosi
    let sdi = peripherals.pins.gpio13; // Miso == 13, not sure if sdi = miso
    let cs = peripherals.pins.gpio15;

    let config = config::Config::new()
        .baudrate(24.MegaHertz().into())
        .data_mode(MODE_3);

    
    let spi_device = SpiDeviceDriver::new_single(
        spi,
        sclk,
        sdo,
        Some(sdi),
        Some(cs),
        &SpiDriverConfig::new(),
        &config,
    )?;

    let vcom = 2300; // taken from https://github.com/m5stack/M5EPD/blob/0e63f701929ca033f12233633ae8a395f5cb5ef1/src/M5EPD_Driver.cpp#L64
    let driver = it8951::interface::IT8951SPIInterface::new(spi_device, busy, rst, Delay::new_default());
    let mut epd = it8951::IT8951::new(driver).init(vcom).unwrap();

    println!(
        "Reset and initalized E-Ink Display: \n\r {:?}",
        epd.get_dev_info()
    );

    // Draw a filled square
    Rectangle::new(Point::new(50, 350), Size::new(20, 20))
        .into_styled(PrimitiveStyle::with_fill(Gray4::BLACK))
        .draw(&mut epd)
        .unwrap();

    Rectangle::new(Point::new(0, 1000), Size::new(200, 200))
        .into_styled(PrimitiveStyle::with_fill(Gray4::new(8)))
        .draw(&mut epd)
        .unwrap();

    // Draw centered text.
    let text = "IT8951 Driver Example";
    embedded_graphics::text::Text::with_alignment(
        text,
        epd.bounding_box().center() + Point::new(0, 15),
        embedded_graphics::mono_font::MonoTextStyle::new(
            &embedded_graphics::mono_font::iso_8859_1::FONT_9X18_BOLD,
            Gray4::new(11),
        ),
        embedded_graphics::text::Alignment::Center,
    )
    .draw(&mut epd)
    .unwrap();

    epd.display(it8951::WaveformMode::GL16).unwrap();

    epd.sleep().unwrap();

    Ok(())
}