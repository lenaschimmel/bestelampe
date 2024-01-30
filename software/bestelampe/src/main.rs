use std::num::NonZeroU32;
use std::thread;
use std::time::Duration;

use enumset::EnumSet;
use esp_idf_hal::prelude::*;
use esp_idf_hal::gpio::{AnyIOPin, InputOutput, InterruptType, Pin, PinDriver};
use esp_idf_hal::ledc::config::TimerConfig;
use esp_idf_hal::ledc::{LedcDriver, LedcTimerDriver, LEDC};
use esp_idf_hal::task::notification::Notification;
use esp_idf_hal::timer::{config, TimerDriver};
use esp_idf_hal::units::Hertz;
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::log::EspLogger;
use esp_idf_hal::{peripherals, uart};
use esp_idf_hal::delay::TickType;
use esp_idf_hal::uart::config::*;
use esp_idf_hal::delay::Delay;

use ds18b20::{Ds18b20, Resolution, SensorData};
use esp_idf_sys::EspError;
use one_wire_bus::{OneWire, OneWireError};

mod pwm;
use crate::pwm::{Pwm, XyColor};

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    EspLogger::initialize_default();

    let mut peripherals: Peripherals = Peripherals::take()?;

    //return dump_sensor(&mut peripherals);
    //return get_sensor_timing();
    //return test_leds();
    //return test_temperature_sensor();
    let temperature_thread = thread::spawn(|| {
        test_temperature_sensor(PinDriver::input_output(peripherals.pins.gpio15).expect("Should be able to take Gpio15 for temperature measurement."));
    });
    
    //let led_thread = thread::spawn(|| {
        let ledc = peripherals.ledc;
        let pin_r  : AnyIOPin = peripherals.pins.gpio10.into();
        let pin_g  : AnyIOPin = peripherals.pins.gpio11.into();
        let pin_b  : AnyIOPin = peripherals.pins.gpio18.into();
        let pin_cw : AnyIOPin = peripherals.pins.gpio19.into();
        let pin_ww : AnyIOPin = peripherals.pins.gpio20.into();
        let pin_a  : AnyIOPin = peripherals.pins.gpio21.into();
    
        test_leds(ledc, pin_r, pin_g, pin_b, pin_cw, pin_ww, pin_a).expect("LEDs should just work.");
    //});

    //led_thread.join();
    //temperature_thread.join();

    Ok(())
}

fn dump_sensor(peripherals: &mut Peripherals) -> anyhow::Result<()> {
    let tx = &mut peripherals.pins.gpio16;
    let rx = &mut peripherals.pins.gpio17;

    println!("Connecting to GPIO 17 to sample the sensor");

    std::thread::sleep(core::time::Duration::from_millis(500));

    // Initialize config manually, because `uart::config::Config::default()`
    // crashes on ESP32-C6 due to an invalid SourceClock.
    let config = uart::config::Config {
        baudrate: Hertz(115_200),
        //baudrate: Hertz(5_830),
        data_bits: DataBits::DataBits8,
        parity: Parity::ParityNone,
        stop_bits: StopBits::STOP1,
        flow_control: FlowControl::None,
        flow_control_rts_threshold: 122,
        source_clock: SourceClock::Crystal,
        intr_flags: EnumSet::EMPTY,
        event_config: EventConfig::new(),
        rx_fifo_size: 128 * 2,
        tx_fifo_size: 128 * 2,
        queue_size: 0,
        _non_exhaustive: (),
    };

    let mut uart: uart::UartDriver = uart::UartDriver::new(
        &mut peripherals.uart1,
        tx,
        rx,
        Option::<AnyIOPin>::None,
        Option::<AnyIOPin>::None,
        &config
    ).unwrap();

    println!("Try to read stuff...");
    loop {
        let mut buf = [0_u8; 1];
        uart.read(&mut buf, TickType::from(Duration::from_millis(500)).ticks()).unwrap();
        println!("read 0x{:02x}", buf[0]);
    }

    return Ok(());
}


// Failed code to determine the bit rate of the presence sensor
//
// Using it, I got:
// 39.65 Million timer ticks = about 1 Second (used stop watch to count 20 packets)
// 6800 timer ticks = 1 Bit
// Factor 5830, e.g. this should be the baud rate
// I think I measured the time it takes to print the debug output to the console, instead of measuring the input signal timing.
// fn get_sensor_timing(peripherals: &mut Peripherals) -> anyhow::Result<()> {
//     let mut sensor_in_pin = PinDriver::input(peripherals.pins.gpio17)?;
//     sensor_in_pin.set_interrupt_type(InterruptType::AnyEdge)?;

//     let timer_conf = config::Config::new().auto_reload(false).divider(2);
//     let mut timer = TimerDriver::new(peripherals.timer10, &timer_conf)?;


//     // Configures the notification
//     let notification = Notification::new();
//     let notifier = notification.notifier();

//     timer.enable(true)?;

//     // Safety: make sure the `Notification` object is not dropped while the subscription is active
//     unsafe {
//         sensor_in_pin.subscribe(move || {
//             let time = timer.counter().unwrap_or_default();
//             notifier.notify_and_yield(NonZeroU32::new((1 + time) as u32).unwrap());
//         })?;
//     }

//     loop {
//         // enable_interrupt should also be called after each received notification from non-ISR context
//         sensor_in_pin.enable_interrupt()?;
//         let time = notification.wait(esp_idf_svc::hal::delay::BLOCK);
//         match time {
//             Some(t) => {
//                 let outer_time = 34; // timer.counter().unwrap_or(1200);
//                 println!("t: {} or {}", t, outer_time);
//             },
//             None => print!("t: error")
//         }
//         //notification.wait(esp_idf_svc::hal::delay::BLOCK);
//         println!("Int!");
        
//     }
// }

fn test_leds(
    ledc: LEDC,
    pin_r:  AnyIOPin,
    pin_g:  AnyIOPin,
    pin_b:  AnyIOPin,
    pin_cw: AnyIOPin,
    pin_ww: AnyIOPin,
    pin_a:  AnyIOPin,
) -> anyhow::Result<()> {

    let timer_driver: LedcTimerDriver<'_> = LedcTimerDriver::new(
        ledc.timer0, 
        &TimerConfig::default().frequency(4600.Hz().into()).resolution(esp_idf_hal::ledc::Resolution::Bits14)
    ).expect("Get LEDC timer.");
    
    let driver_0 = LedcDriver::new(ledc.channel0, &timer_driver, pin_r ).expect("Get LEDC driver.");
    let driver_1 = LedcDriver::new(ledc.channel1, &timer_driver, pin_g ).expect("Get LEDC driver.");
    let driver_2 = LedcDriver::new(ledc.channel2, &timer_driver, pin_b ).expect("Get LEDC driver.");
    let driver_3 = LedcDriver::new(ledc.channel3, &timer_driver, pin_cw).expect("Get LEDC driver.");
    let driver_4 = LedcDriver::new(ledc.channel4, &timer_driver, pin_ww).expect("Get LEDC driver.");
    let driver_5 = LedcDriver::new(ledc.channel5, &timer_driver, pin_a ).expect("Get LEDC driver.");

    println!("Before LED main loop...");
    std::thread::sleep(core::time::Duration::from_millis(500));
    
    let mut time: f32 = 0.0;
    let mut pwm = Pwm::new(
        driver_0,
        driver_1,
        driver_2,
        driver_3,
        driver_4,
        driver_5,
    )?;

    loop {
        std::thread::sleep(core::time::Duration::from_millis(50));
        time += 50.0;

        let mut t =  (time % 20_000.0) / 10_000.0;
        if t > 1.0 {
            t = 2.0 - t;
        }
        let x = t * 0.8 + 0.2;

        let mired = (1.0-x).powf(2.5) * 950.0 + 50.0;
        let temperature = 1_000_000.0 / mired;
        let brightness = 6.0; 
        // time / 20_000.0 + 17.0;
        if((time as i64) % 1000 == 0) {
            println!("Time: {}, brightness: {:2.4}, temperature: {:5.0}", time, brightness, temperature);
        }
        pwm.set_temperature_and_brightness(temperature, brightness).ok();
    }
    
}


fn test_temperature_sensor<PinType: Pin>(one_wire_pin: PinDriver<'_, PinType, InputOutput>)  -> anyhow::Result<()> {
    println!("Before temperature sensor init...");
    let mut delay = Delay::new_default();

    let mut one_wire_bus = OneWire::new(one_wire_pin).unwrap();

    // initiate a temperature measurement for all connected devices
    ds18b20::start_simultaneous_temp_measurement(&mut one_wire_bus, &mut delay).expect("Start temp measurement");

    // wait until the measurement is done. This depends on the resolution you specified
    // If you don't know the resolution, you can obtain it from reading the sensor data,
    // or just wait the longest time, which is the 12-bit resolution (750ms)
    
    // Don't block, use non-blocking sleep instead
    // Resolution::Bits12.delay_for_measurement_time(&mut delay);
    std::thread::sleep(core::time::Duration::from_millis(Resolution::Bits12.max_measurement_time_millis() as u64));

    let mut sensors: Vec<Ds18b20> = Vec::new();

    println!("Before temperature sensor search loop...");
    // iterate over all the devices, and report their temperature
    let mut search_state = None;
    loop {
        if let Some((device_address, state)) = one_wire_bus.device_search(search_state.as_ref(), false, &mut delay).expect("Device search") {
            search_state = Some(state);
            if device_address.family_code() != ds18b20::FAMILY_CODE {
                // skip other devices
                continue; 
            }
            // You will generally create the sensor once, and save it for later
            let sensor = Ds18b20::new::<OneWireError<EspError>>(device_address).expect("Create device by address");
            
            // contains the read temperature, as well as config info such as the resolution used
            let sensor_data = sensor.read_data(&mut one_wire_bus, &mut delay).expect("Read sensor data");
            println!("Device at {:?} is {}°C. Resolution is {:?}.", device_address, sensor_data.temperature, sensor_data.resolution);
            sensors.push(sensor);
        } else {
            break;
        }
    }

    println!("Before continous temperature sensor measurement loop...");
    loop {
        // Start all measurements
        for sensor in &sensors {
            sensor.start_temp_measurement(&mut one_wire_bus, &mut delay).expect("Start single measurement");
        }
        // Wait for the longest possible measurement time
        //std::thread::sleep(core::time::Duration::from_millis(Resolution::Bits12.max_measurement_time_millis() as u64));

        // Wait two seconds. We don't need more frequent measurements.
        std::thread::sleep(core::time::Duration::from_millis(2000));

        // Read all measurements
        for sensor in &sensors {
            let sensor_data = sensor.read_data(&mut one_wire_bus, &mut delay).expect("Read sensor data");
            println!("Device at {:?} is {}°C.", sensor.address(), sensor_data.temperature);
        }
    }

    Ok(())
}