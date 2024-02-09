use std::thread;
use std::time::Duration;
use std::sync::{Arc, RwLock};

use enumset::EnumSet;
use esp_idf_hal::prelude::*;
use esp_idf_hal::gpio::{AnyIOPin, InputOutput, Pin, PinDriver};
use esp_idf_hal::ledc::config::TimerConfig;
use esp_idf_hal::ledc::{LedcDriver, LedcTimerDriver, LEDC};
use esp_idf_hal::units::Hertz;
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::log::EspLogger;
use esp_idf_hal::{uart, modem::Modem};
use esp_idf_hal::delay::TickType;
use esp_idf_hal::uart::config::*;
use esp_idf_hal::delay::Delay;

use ds18b20::{Ds18b20, Resolution};
use esp_idf_sys::EspError;
use one_wire_bus::{OneWire, OneWireError};
use prisma::Lerp;

use core::convert::TryInto;
use anyhow::{ Result, anyhow };

use esp_idf_svc::{
    io::{Read, Write},
    http::Method,
    eventloop::EspSystemEventLoop,
    http::server::EspHttpServer,
    nvs::EspDefaultNvsPartition,
    wifi::AuthMethod,
    wifi::{BlockingWifi, EspWifi},
};

use log::*;

use serde::Deserialize;

mod pwm;
use crate::pwm::Pwm;


fn main() -> ! {
    esp_idf_svc::sys::link_patches();
    EspLogger::initialize_default();
    
    let thermal: Arc<RwLock<f32>> = Arc::new(RwLock::new(0.0));
    let light_temperature_target: Arc<RwLock<f32>> = Arc::new(RwLock::new(3000.0));
    let light_brightness_target: Arc<RwLock<f32>> = Arc::new(RwLock::new(2.0));

    let peripherals: Peripherals = Peripherals::take().expect("Need Peripherals.");

    //return dump_sensor(&mut peripherals);
    //return get_sensor_timing();
    //return test_leds();
    //return test_temperature_sensor();
    // let _temperature_thread = thread::spawn(|| {
    //     test_temperature_sensor(PinDriver::input_output(peripherals.pins.gpio15).expect("Should be able to take Gpio15 for temperature measurement."));
    // });
    
    let light_temperature_target_clone = light_temperature_target.clone();
    let light_brightness_target_clone = light_brightness_target.clone();
    let _led_thread = thread::spawn(|| {
        let ledc = peripherals.ledc;
        let pin_r  : AnyIOPin = peripherals.pins.gpio10.into();
        let pin_g  : AnyIOPin = peripherals.pins.gpio11.into();
        let pin_b  : AnyIOPin = peripherals.pins.gpio18.into();
        let pin_cw : AnyIOPin = peripherals.pins.gpio19.into();
        let pin_ww : AnyIOPin = peripherals.pins.gpio20.into();
        let pin_a  : AnyIOPin = peripherals.pins.gpio21.into();
    
        test_leds(ledc, pin_r, pin_g, pin_b, pin_cw, pin_ww, pin_a, light_temperature_target_clone, light_brightness_target_clone).expect("LEDs should just work.");
    });

    let _wifi_thread = thread::spawn(|| {
        test_wifi(peripherals.modem, light_temperature_target, light_brightness_target, true);
    });

    println!("Entering infinite loop in main thread...");
    loop {}
}

fn test_presence_sensor(peripherals: &mut Peripherals) -> ! {
    let tx = &mut peripherals.pins.gpio16;
    let rx = &mut peripherals.pins.gpio17;

    println!("Connecting to GPIO 17 to sample the sensor");

    std::thread::sleep(core::time::Duration::from_millis(500));

    // Initialize config manually, because `uart::config::Config::default()`
    // crashes on ESP32-C6 due to an invalid SourceClock.
    let config = uart::config::Config {
        baudrate: Hertz(115_200),
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

    let uart: uart::UartDriver = uart::UartDriver::new(
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
}

fn test_leds(
    ledc: LEDC,
    pin_r:  AnyIOPin,
    pin_g:  AnyIOPin,
    pin_b:  AnyIOPin,
    pin_cw: AnyIOPin,
    pin_ww: AnyIOPin,
    pin_a:  AnyIOPin,
    light_temperature_target: Arc<RwLock<f32>>,
    light_brightness_target: Arc<RwLock<f32>>,
) -> Result<()> {

    // FIXME For ESP32-C6 `Resolution::Bits14` is the largest enum that is defined. But the C6 supports resolutions up to Bits20.
    // I'd like to use Bits16 and 1000 Hz here, which should be okay.
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
    
    let mut target_brightness = 0.0;
    let mut target_temperature = 0.0;
    {
        target_brightness = *(light_brightness_target.read().unwrap());
        target_temperature = *(light_temperature_target.read().unwrap());
    }
    let mut brightness = target_brightness;
    let mut temperature = target_temperature;

    let lerp_speed = 0.01;

    loop {
        std::thread::sleep(core::time::Duration::from_millis(50));
        time += 50.0;
        
        {
            target_brightness = *(light_brightness_target.read().unwrap());
            target_temperature = *(light_temperature_target.read().unwrap());
        }
        
        brightness = brightness.lerp(&target_brightness, lerp_speed);
        temperature = temperature.lerp(&target_temperature, lerp_speed);

        pwm.set_temperature_and_brightness(temperature, brightness)?;
    }
    
}


fn test_temperature_sensor<PinType: Pin>(one_wire_pin: PinDriver<'_, PinType, InputOutput>, thermal: Arc<RwLock<f32>>)  -> ! {
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
        if let Ok(Some((device_address, state))) = one_wire_bus.device_search(search_state.as_ref(), false, &mut delay) {
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

    println!("Before continuos temperature sensor measurement loop...");
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
        let mut thermal_write = thermal.write().unwrap();
        for sensor in &sensors {
            let sensor_data = sensor.read_data(&mut one_wire_bus, &mut delay).expect("Read sensor data");
            *thermal_write = sensor_data.temperature;
            println!("Device at {:?} is {}°C.", sensor.address(), sensor_data.temperature);
        }
    }
}

#[derive(Deserialize)]
struct FormData {
    brightness: f32,
    temperature: f32,
}

fn test_wifi(
    modem: Modem,
    light_temperature_target: Arc<RwLock<f32>>,
    light_brightness_target: Arc<RwLock<f32>>,
    as_access_point: bool
) -> Result<()> {
    // this used to be the main method of an example program

    let mut server: EspHttpServer<'_>;
    if (as_access_point) {
        server = create_access_point_server(modem)?;
    } else {
        server = create_station_server(modem)?;
    }
    println!("Created the server. Attaching handlers.");

    server.fn_handler::<anyhow::Error, _>("/", Method::Get, |req| {
        req.into_ok_response()?.write_all(INDEX_HTML.as_bytes()).map(|_| ())?;
        return Ok(());
    })?;

    server.fn_handler::<anyhow::Error, _>("/post", Method::Post, |mut req| {
        let len = req.header("Content-Length") .and_then(|v| v.parse::<u64>().ok()).unwrap_or(0) as usize;

        if len > MAX_LEN {
            req.into_status_response(413)?
                .write_all("Request too big".as_bytes())?;
            return Ok(());
        }

        let mut buf = vec![0; len];
        req.read_exact(&mut buf)?;
        let mut resp = req.into_ok_response()?;

        if let Ok(form) = serde_json::from_slice::<FormData>(&buf) {
            write!(
                resp,
                "Set color temperature to {}K, brightness to {}...",
                form.temperature, form.brightness
            )?;
            *light_brightness_target.write().unwrap() = form.brightness;
            *light_temperature_target.write().unwrap() = form.temperature;
            
        } else {
            resp.write_all("JSON error".as_bytes())?;
        }

        Ok(())
    })?;

    println!("Handlers attached.");


    loop {
        println!("Inside server keep-alive loop.");
        std::thread::sleep(core::time::Duration::from_millis(2000));
    }

    println!("The server is over now.");

    // Keep server running beyond when main() returns (forever)
    // Do not call this if you ever want to stop or access it later.
    // Otherwise you can either add an infinite loop so the main task
    // never returns, or you can move it to another thread.
    // https://doc.rust-lang.org/stable/core/mem/fn.forget.html
    core::mem::forget(server);

    // Main task no longer needed, free up some memory
    Ok(())
}

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
}

static INDEX_HTML: &str = include_str!("http_server_page.html");

// Max payload length
const MAX_LEN: usize = 128;

// Need lots of stack to parse JSON
const STACK_SIZE: usize = 10240;

// Wi-Fi channel, between 1 and 11
const CHANNEL: u8 = 11;

fn create_station_server(modem: Modem) -> Result<EspHttpServer<'static>> {
    println!("Inside 'create_station_server'...");
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(modem, sys_loop.clone(), Some(nvs))?,
        sys_loop,
    )?;

    let wifi_configuration = esp_idf_svc::wifi::Configuration::Client(esp_idf_svc::wifi::ClientConfiguration {
        ssid: CONFIG.wifi_ssid.try_into().or(Err(anyhow!("Invalid SSID config.")))?,
        password: CONFIG.wifi_psk.try_into().or(Err(anyhow!("Invalid PSK config.")))?,
        auth_method: AuthMethod::WPA2Personal,
        ..Default::default()
    });
    wifi.set_configuration(&wifi_configuration)?;
    wifi.start()?;
    wifi.connect()?;
    wifi.wait_netif_up()?;

    info!(
        "Joined Wi-Fi with WIFI_SSID `{}` and WIFI_PASS `{}`",
        CONFIG.wifi_ssid, CONFIG.wifi_psk
    );

    let server_configuration = esp_idf_svc::http::server::Configuration {
        stack_size: STACK_SIZE,
        ..Default::default()
    };

    core::mem::forget(wifi);

    return EspHttpServer::new(&server_configuration).or(Err(anyhow!("Could not create server.")));
}

fn create_access_point_server(modem: Modem) -> Result<EspHttpServer<'static>> {
    println!("Inside 'create_access_point_server'...");
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(modem, sys_loop.clone(), Some(nvs))?,
        sys_loop,
    )?;

    let wifi_configuration = esp_idf_svc::wifi::Configuration::AccessPoint(esp_idf_svc::wifi::AccessPointConfiguration {
        ssid: CONFIG.wifi_ssid.try_into().or(Err(anyhow!("Invalid SSID config.")))?,
        password: CONFIG.wifi_psk.try_into().or(Err(anyhow!("Invalid PSK config.")))?,
        ssid_hidden: false,
        auth_method: AuthMethod::WPA2Personal,
        channel: CHANNEL,
        ..Default::default()
    });
    wifi.set_configuration(&wifi_configuration)?;
    wifi.start()?;
    wifi.wait_netif_up()?;

    info!(
        // This will output the configuration as it is read, e.g. currently empty strings.
        "Created Wi-Fi with WIFI_SSID `{}` and WIFI_PASS `{}`",
        CONFIG.wifi_ssid, CONFIG.wifi_psk
    );

    let server_configuration = esp_idf_svc::http::server::Configuration {
        stack_size: STACK_SIZE,
        ..Default::default()
    };

    core::mem::forget(wifi);

    return EspHttpServer::new(&server_configuration).or(Err(anyhow!("Could not create server.")));
}