use std::time::Duration;
use std::sync::{Arc, RwLock};
use ::function_name::named;

use enumset::EnumSet;
use esp_idf_hal::{
    prelude::*,
    delay::TickType,
    gpio::AnyIOPin,
    uart, 
    uart::{UART1, config::*},
};

use log::*;
use mr24hpc1::{mr_parser, Frame, HumanPresence};


#[named]
pub fn test_presence_sensor(
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
                            println!("Motion: {:?}", motion);
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
