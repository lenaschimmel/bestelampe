use std::error::Error;

use esp_idf_hal::{
    prelude::*,
    delay::Delay,
    gpio::*,
    spi::*,
    peripherals::Peripherals,
};

use embedded_hal::spi::MODE_0;

use embedded_graphics::{
    pixelcolor::Gray4,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};

fn main() -> Result<(), Box<dyn Error>> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    println!("Programm is actually running.");

    let peripherals = Peripherals::take().unwrap();

    let busy = PinDriver::input(peripherals.pins.gpio27).unwrap();
    let mut main_power  = PinDriver::output(peripherals.pins.gpio2).unwrap(); // MAIN_PWR_PINM
    let rst  = PinDriver::output(peripherals.pins.gpio19).unwrap(); // This is one of the grove ports. The display driver wants a reset pin, but no GPIO is connected!

    main_power.set_high().unwrap();

    let spi = peripherals.spi2;
    let sclk : AnyIOPin = peripherals.pins.gpio14.into();
    let sdo  : AnyIOPin = peripherals.pins.gpio12.into(); // GPIO12 leads to pin 123 which is an input pin on the it8951
    let sdi  : AnyIOPin = peripherals.pins.gpio13.into(); // GPIO13 leads to pin 124 which is an output pin on the it8951
    let cs   : AnyIOPin = peripherals.pins.gpio15.into();

    let config = config::Config::new()
        .baudrate(MegaHertz(10).into())
        .data_mode(MODE_0);

    
    let spi_device = SpiDeviceDriver::new_single(
        spi,
        sclk,
        sdo,
        Some(sdi),
        Some(cs),
        &SpiDriverConfig::new(),
        &config,
    )?;

    println!("SPI is initialized, driver will follow...");

    let vcom = 2300; // taken from https://github.com/m5stack/M5EPD/blob/0e63f701929ca033f12233633ae8a395f5cb5ef1/src/M5EPD_Driver.cpp#L64
    let driver = it8951::interface::IT8951SPIInterface::new(spi_device, busy, rst, Delay::new_default());
    println!("driver is done, now creating epd...");
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