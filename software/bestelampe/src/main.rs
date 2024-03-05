use std::thread;
use std::time::Duration;
use std::sync::{Arc, RwLock};

use enumset::EnumSet;

use esp_idf_hal::prelude::*;

use esp_idf_hal::{uart, modem::Modem};
use esp_idf_hal::delay::{Delay, TickType};
use esp_idf_hal::gpio::{AnyIOPin, InputOutput, Pin, PinDriver};
use esp_idf_hal::i2c::*;
use esp_idf_hal::ledc::{LedcDriver, LedcTimerDriver, LEDC, config::TimerConfig};
use esp_idf_hal::uart::{UART1, config::*};

use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    http::Method,
    http::server::EspHttpServer,
    io::{Read, Write},
    log::EspLogger,
    nvs::EspDefaultNvsPartition,
    sntp,
    wifi::{BlockingWifi, EspWifi, AuthMethod, WifiDriver},
    netif::{EspNetif, NetifConfiguration, NetifStack},
};

use esp_idf_sys::EspError;

use chrono::{Utc, offset::Local};
use chrono_tz::Tz;

use one_wire_bus::{OneWire, OneWireError};
use prisma::Lerp;

use ds18b20::{Ds18b20, Resolution};
use anyhow::{ Result, anyhow };
use heapless::String;


use veml6040::{Veml6040, IntegrationTime, MeasurementMode};

use log::*;

use serde::Deserialize;

use ::function_name::named;

use simple_error::{SimpleError, try_with};

use mr24hpc1::{mr_parser, Frame, HumanPresence, Motion};

mod pwm;
use crate::pwm::Pwm;

const LIGHT_SENSOR_ADDRESS: u8 = 0x10;

#[named]
fn main() -> ! {
    esp_idf_svc::sys::link_patches();
    EspLogger::initialize_default();
    debug!(target: function_name!(), "Logger initialized.");
    
    // Thread safe globals for communication across tasks
    let thermal: Arc<RwLock<f32>> = Arc::new(RwLock::new(0.0));
    let light_temperature_target: Arc<RwLock<f32>> = Arc::new(RwLock::new(3000.0));
    let light_brightness_target: Arc<RwLock<f32>> = Arc::new(RwLock::new(2.0));
    let light_dim_speed: Arc<RwLock<f32>> = Arc::new(RwLock::new(0.01));

    let peripherals: Peripherals = Peripherals::take().expect("Need Peripherals.");

    // Light sensor
    let i2c = peripherals.i2c0;
    let _light_sensor_thread = thread::spawn(|| {
        test_light_sensor(i2c, peripherals.pins.gpio6.into(), peripherals.pins.gpio7.into()).unwrap_or_default();
        error!(target: function_name!(), "Light sensor has ended :(");
    });

    // Temperature sensor
    let _temperature_thread = thread::spawn(|| {
        let pin_driver = PinDriver::input_output(peripherals.pins.gpio15).expect("Should be able to take Gpio15 for temperature measurement.");
        test_temperature_sensor(pin_driver, thermal);
    });

    // Presence sensor
    let light_brightness_target_clone_for_presence = light_brightness_target.clone();
    let _presence_thread = thread::spawn(|| {
        test_presence_sensor(
            peripherals.pins.gpio17.into(), 
            peripherals.pins.gpio16.into(), 
            peripherals.uart1,
            light_brightness_target_clone_for_presence);
    });
    
    // LED control
    let light_temperature_target_clone = light_temperature_target.clone();
    let light_brightness_target_clone = light_brightness_target.clone();
    let light_dim_speed_clone = light_dim_speed.clone();
    let _led_thread = thread::spawn(|| {
        let ledc = peripherals.ledc;
        let pin_r  : AnyIOPin = peripherals.pins.gpio10.into();
        let pin_g  : AnyIOPin = peripherals.pins.gpio11.into();
        let pin_b  : AnyIOPin = peripherals.pins.gpio18.into();
        let pin_cw : AnyIOPin = peripherals.pins.gpio19.into();
        let pin_ww : AnyIOPin = peripherals.pins.gpio20.into();
        let pin_a  : AnyIOPin = peripherals.pins.gpio21.into();
    
        test_leds(ledc, pin_r, pin_g, pin_b, pin_cw, pin_ww, pin_a, light_temperature_target_clone, light_brightness_target_clone, light_dim_speed_clone).expect("LEDs should just work.");
    });

    // Wifi & web interface server
    let _wifi_thread = thread::spawn(|| {
        start_wifi(peripherals.modem, false).unwrap();
        let _sntp = sntp::EspSntp::new_default().unwrap();
        info!(target: function_name!(), "SNTP initialized");
        run_server(light_temperature_target, light_brightness_target, light_dim_speed).unwrap();
    });

    let tz: Tz = CONFIG.time_zone.parse().unwrap();

    // Keep the main thread alive
    info!(target: function_name!(), "Entering infinite loop in main thread...");
    loop {
        let now = Utc::now();
        let local_now = now.with_timezone(&tz);
        info!("Current time: {:?}", local_now); 
        std::thread::sleep(core::time::Duration::from_millis(1000));
    }
}

const DARK_THRESHOLD_SOFT: u16 = 500;
const DARK_THRESHOLD_HARD: u16 = 10;
const BRIGHT_THRESHOLD_SOFT: u16 = 20_000;
const BRIGHT_THRESHOLD_HARD: u16 = 64_000;

const INTEGRATION_TIMES: [IntegrationTime; 6] = [
    IntegrationTime::_40ms,
    IntegrationTime::_80ms,
    IntegrationTime::_160ms,
    IntegrationTime::_320ms,
    IntegrationTime::_640ms,
    IntegrationTime::_1280ms,
];

const WAIT_TIMES: [u16; 6] = [
    40 + 40,
    40 + 80,
    40 + 160,
    40 + 320,
    40 + 640,
    40 + 1280,
];

const SENSITIVITIES: [f32; 6] = [
    0.25168,
    0.12584,
    0.06292,
    0.03146,
    0.01573,
    0.007865,
];

#[named]
fn test_light_sensor(i2c: I2C0, scl: AnyIOPin, sda: AnyIOPin) -> Result<()> {
    let config = I2cConfig::new().baudrate(100.kHz().into()).scl_enable_pullup(false).sda_enable_pullup(false);
    let mut i2c = I2cDriver::new(i2c, sda, scl, &config)?;

    info!(target: function_name!(), "Creating the Veml device...");

    let mut sensor = Veml6040::new(i2c);
    info!(target: function_name!(), "Trying to enable and set config...");
    sensor.enable().unwrap();

    let mut integration_time_index = 3;
    sensor.set_integration_time(INTEGRATION_TIMES[integration_time_index]).unwrap();
    sensor.set_measurement_mode(MeasurementMode::Manual).unwrap();

    let mut index_changed = false;

    info!(target: function_name!(), "Reading values...");
    loop {
        // This is my attempt at something like try-catch in Rust, so that an error in the
        // loop iteration just skips to the next iteration. Is there an easier way?
        let mut iteration = || -> Result<(), SimpleError> {
            sensor.trigger_measurement().map_err(|_e| SimpleError::new("Failed triggering measurement."))?;
            let min_wait_time = if index_changed { 0 } else { 1000 };
            let wait_time = u16::max(min_wait_time, WAIT_TIMES[integration_time_index]);
            index_changed = false;

            std::thread::sleep(core::time::Duration::from_millis(wait_time as u64));

            let reading = sensor.read_all_channels().map_err(|_e| SimpleError::new("Failed reading all channels for the first time."))?;
            
            // println!("Combined measurements: red = {}, green = {}, blue = {}, white = {}",  
            //    reading.red, reading.green, reading.blue, reading.white);

            let green = reading.green;

            if green < DARK_THRESHOLD_HARD {
                debug!(target: function_name!(), "Too dark for accurate lux measurement");
            } else if green > BRIGHT_THRESHOLD_HARD {
                debug!(target: function_name!(), "Too bright for accurate lux measurement");
            } else {
                let lux = green as f32 * SENSITIVITIES[integration_time_index];
            
                let blue = reading.blue;
                let red = reading.red;
                if red > DARK_THRESHOLD_HARD && red < BRIGHT_THRESHOLD_HARD 
                    && blue > DARK_THRESHOLD_HARD && blue < BRIGHT_THRESHOLD_HARD 
                {
                    let ccti = (red as f32 - blue as f32) / (green as f32) + 0.5;
                    let cct = 4278.6 * ccti.powf(-1.2455);
                    info!(target: function_name!(), "Brightness: {} lx, color temperature: {} K", lux, cct);
                } else {
                    info!(target: function_name!(), "Brightness: {} lx, color temperature unknown", lux);
                }
            }
            
            if green < DARK_THRESHOLD_SOFT && integration_time_index < 5 {
                integration_time_index += 1;
                sensor.set_integration_time(INTEGRATION_TIMES[integration_time_index]).map_err(|_e| SimpleError::new("Failed setting integration time."))?;
                debug!(target: function_name!(), "Switching to longer integration time {:?}...", INTEGRATION_TIMES[integration_time_index]);
                index_changed = true;
            }
            if green > BRIGHT_THRESHOLD_SOFT  && integration_time_index > 0 {
                integration_time_index -= 1;
                sensor.set_integration_time(INTEGRATION_TIMES[integration_time_index]).map_err(|_e| SimpleError::new("Failed setting integration time."))?;
                debug!(target: function_name!(), "Switching to shorter integration time {:?}...", INTEGRATION_TIMES[integration_time_index]);
                index_changed = true;
            }     
            Ok(())   
        };
        if let Err(err) = iteration() {
            warn!(target: function_name!(), "Error in sensor loop iteration: {}", err);
            std::thread::sleep(core::time::Duration::from_millis(750));
        }
    }

    return Ok(());
}

#[named]
fn test_presence_sensor(
    pin_rx: AnyIOPin,
    pin_tx: AnyIOPin,
    uart_device: UART1, 
    light_brightness_target: Arc<RwLock<f32>>,
) -> ! {
    info!(target: function_name!(), "Connecting to GPIO 17 to sample the sensor");

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
        uart_device,
        pin_tx,
        pin_rx,
        Option::<AnyIOPin>::None,
        Option::<AnyIOPin>::None,
        &config
    ).unwrap();

    info!(target: function_name!(), "Try to read stuff...");
    loop {
        let mut buf = [0_u8; 20];
        let len = uart.read(&mut buf, TickType::from(Duration::from_millis(50)).ticks()).unwrap();
        if len >= 9 {
            match mr_parser(&buf[0..len]) {
                Ok((_, frame)) => {
                    info!(target: function_name!(), "Parsed presence data: {:?}", frame);
                    match frame {
                        // Frame::HumanPresenceReport(HumanPresence::BodyMovementParameter(movement)) => {
                        //     *light_brightness_target.write().unwrap() = movement as f32;
                        // },
                        Frame::HumanPresenceReport(HumanPresence::MotionInformation(motion)) => {
                            // *light_brightness_target.write().unwrap() = match motion {
                            //     Motion::None => 0.0,
                            //     Motion::Motionless => 0.5,
                            //     Motion::Active => 1.5,
                            // }
                        },
                        _ => {

                        }
                    }
                },
                Err(_) => {
                    warn!(target: function_name!(), "Error while parsing presence data '{:x?}'", buf);
                },
            }
        } else if len > 9 {
            warn!(target: function_name!(), "Short resence data '{:x?}'", buf);
        }
    }
}

#[named]
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
    light_dim_speed: Arc<RwLock<f32>>,
) -> Result<()> {

    // FIXME For ESP32-C6 `Resolution::Bits14` is the largest enum that is defined. But the C6 supports resolutions up to Bits20.
    // I'd like to use Bits16 and 1000 Hz here, which should be okay.
    let timer_driver: LedcTimerDriver<'_> = LedcTimerDriver::new(
        ledc.timer0, 
        &TimerConfig::default().frequency(2400.Hz().into()).resolution(esp_idf_hal::ledc::Resolution::Bits14)
    ).expect("Get LEDC timer.");
    
    let driver_0 = LedcDriver::new(ledc.channel0, &timer_driver, pin_r ).expect("Get LEDC driver.");
    let driver_1 = LedcDriver::new(ledc.channel1, &timer_driver, pin_g ).expect("Get LEDC driver.");
    let driver_2 = LedcDriver::new(ledc.channel2, &timer_driver, pin_b ).expect("Get LEDC driver.");
    let driver_3 = LedcDriver::new(ledc.channel3, &timer_driver, pin_cw).expect("Get LEDC driver.");
    let driver_4 = LedcDriver::new(ledc.channel4, &timer_driver, pin_ww).expect("Get LEDC driver.");
    let driver_5 = LedcDriver::new(ledc.channel5, &timer_driver, pin_a ).expect("Get LEDC driver.");

    info!(target: function_name!(), "Before LED main loop...");
    std::thread::sleep(core::time::Duration::from_millis(550));
    
    let mut time: f32 = 0.0;
    let mut pwm = Pwm::new(
        driver_0,
        driver_1,
        driver_2,
        driver_3,
        driver_4,
        driver_5,
    )?;
    
    let (mut target_brightness, mut target_temperature, mut dim_speed) = {(
        *(light_brightness_target.read().unwrap()),
        *(light_temperature_target.read().unwrap()),
        *(light_dim_speed.read().unwrap()),

    )};
    let (mut brightness, mut temperature) = (target_brightness, target_temperature);

    let mut count: i32 = 0;
    loop {
        std::thread::sleep(core::time::Duration::from_millis(500));
        time += 50.0;
        count += 1;
        
        {
            target_brightness = *(light_brightness_target.read().unwrap());
            target_temperature = *(light_temperature_target.read().unwrap());
            dim_speed = *(light_dim_speed.read().unwrap());
        }
        
        brightness = brightness.lerp(&target_brightness, dim_speed);
        temperature = temperature.lerp(&target_temperature, dim_speed);

        if count % 40 == 0 {
            info!(target: function_name!(), "Current temp: {}, brightness: {}", temperature, brightness);
        }

        pwm.set_temperature_and_brightness(temperature, brightness)?;
    }
    
}

#[named]
fn test_temperature_sensor<PinType: Pin>(one_wire_pin: PinDriver<'_, PinType, InputOutput>, thermal: Arc<RwLock<f32>>)  -> ! {
    info!(target: function_name!(), "Before temperature sensor init...");
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

    info!(target: function_name!(), "Before temperature sensor search loop...");
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
            match sensor.read_data(&mut one_wire_bus, &mut delay) {
                Ok(sensor_data) => {
                    info!(target: function_name!(), "Device at {:?} is {}°C. Resolution is {:?}.", device_address, sensor_data.temperature, sensor_data.resolution);
                    sensors.push(sensor);
                },
                Err(e) => warn!(target: function_name!(), "Error reading sensor data for the first time: {:?}", e),
            }
            
        } else {
            break;
        }
    }

    info!(target: function_name!(), "Before continuos temperature sensor measurement loop...");
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
            match sensor.read_data(&mut one_wire_bus, &mut delay) {
                Ok(sensor_data) => {
                    *thermal_write = sensor_data.temperature;
                    info!(target: function_name!(), "Device at {:?} is {}°C.", sensor.address(), sensor_data.temperature);
                },
                Err(e) => warn!(target: function_name!(), "Error while reading thermal temperature: {:?}", e),
            }
        }
    }
}

#[derive(Deserialize)]
struct FormData {
    brightness: f32,
    temperature: f32,
    speed: f32,
}

#[named]
fn run_server(
    light_temperature_target: Arc<RwLock<f32>>,
    light_brightness_target: Arc<RwLock<f32>>,
    light_dim_speed: Arc<RwLock<f32>>,
) -> Result<()> {
    
    let server_configuration = esp_idf_svc::http::server::Configuration {
        stack_size: STACK_SIZE,
        ..Default::default()
    };
    let mut server: EspHttpServer<'_> = EspHttpServer::new(&server_configuration).or(Err(anyhow!("Could not create server.")))?;
    info!(target: function_name!(), "Created the server. Attaching handlers.");

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
                "Set color temperature to {}K, brightness to {} with speed {}...",
                form.temperature, form.brightness, form.speed
            )?;
            *light_brightness_target.write().unwrap() = form.brightness;
            *light_temperature_target.write().unwrap() = form.temperature;
            *light_dim_speed.write().unwrap() = form.speed;   
        } else {
            resp.write_all("JSON error".as_bytes())?;
        }

        Ok(())
    })?;

    info!(target: function_name!(), "Handlers attached.");

    loop {
        trace!(target: function_name!(), "Inside server keep-alive loop.");
        std::thread::sleep(core::time::Duration::from_millis(2000));
    }
}

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,   
    #[default("Etc/GMT")]
    time_zone: &'static str,
}

static INDEX_HTML: &str = include_str!("http_server_page.html");

// Max payload length
const MAX_LEN: usize = 128;

// Need lots of stack to parse JSON
const STACK_SIZE: usize = 10240;

// Wi-Fi channel, between 1 and 11
const CHANNEL: u8 = 11;

/// Starts the wifi either as station ("client") or access point.
/// Does not have any retry-loop or error handling.
/// Method returns when the wifi is ready to be used.
#[named]
fn start_wifi(modem: Modem, as_access_point: bool) -> Result<()> {
    info!(target: function_name!(), "Inside 'start_wifi'...");
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let ipv4_client_cfg =
        esp_idf_svc::ipv4::ClientConfiguration::DHCP(esp_idf_svc::ipv4::DHCPClientSettings {
            hostname: Some(heapless::String::<30>::try_from("besteLampe").unwrap()),
            ..Default::default()
        });
    let new_c = NetifConfiguration {
        ip_configuration: esp_idf_svc::ipv4::Configuration::Client(ipv4_client_cfg),
        ..NetifConfiguration::wifi_default_client()
    };

    let esp_wifi = EspWifi::wrap_all(
        WifiDriver::new(
            modem,
            sys_loop.clone(),
            Some(nvs),
        )?,
        EspNetif::new_with_conf(&new_c)?,
        EspNetif::new(NetifStack::Ap)?,
    )?;

    let mut wifi = BlockingWifi::wrap(
        esp_wifi,
        sys_loop,
    )?;

    let wifi_configuration = match as_access_point {
        false => 
            esp_idf_svc::wifi::Configuration::Client(esp_idf_svc::wifi::ClientConfiguration {
            ssid: CONFIG.wifi_ssid.try_into().or(Err(anyhow!("Invalid SSID config.")))?,
            password: CONFIG.wifi_psk.try_into().or(Err(anyhow!("Invalid PSK config.")))?,
            auth_method: AuthMethod::WPA2Personal,
            ..Default::default()
        }),
        true => esp_idf_svc::wifi::Configuration::AccessPoint(esp_idf_svc::wifi::AccessPointConfiguration {
            ssid: CONFIG.wifi_ssid.try_into().or(Err(anyhow!("Invalid SSID config.")))?,
            password: CONFIG.wifi_psk.try_into().or(Err(anyhow!("Invalid PSK config.")))?,
            ssid_hidden: false,
            auth_method: AuthMethod::WPA2Personal,
            channel: CHANNEL,
            ..Default::default()
        })
    };

    info!(target: function_name!(), "Setting configuration...");
    wifi.set_configuration(&wifi_configuration)?;
    info!(target: function_name!(), "Starting...");
    wifi.start()?;
    if !as_access_point {
        info!(target: function_name!(), "Connecting...");
        wifi.connect()?;
    }
    info!(target: function_name!(), "Waiting for netif...");
    wifi.wait_netif_up()?;

    info!(target: function_name!(), 
        "Joined Wi-Fi with WIFI_SSID `{}` and WIFI_PASS `{}` as {}",
        CONFIG.wifi_ssid, CONFIG.wifi_psk, if as_access_point { "access point" } else { "station" }
    );

    core::mem::forget(wifi);

    return Ok(());
}