// SPDX-FileCopyrightText: 2024 Lena Schimmel <mail@lenaschimmel.de>
// SPDX-License-Identifier: CERN-OHL-S-2.0+
// This file is part of besteLampe!.
// 
// besteLampe! is free software: you can redistribute it and/or modify it under the
// terms of the GNU General Public License as published by the Free Software Foundation, 
// either version 3 of the License, or (at your option) any later version.
// 
// besteLampe! is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; 
// without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. 
// See the GNU General Public License for more details.
// 
// You should have received a copy of the GNU General Public License along with besteLampe!.
// If not, see <https://www.gnu.org/licenses/>. 

use crate::prelude::*;

use std::time::Duration;
use std::sync::{Arc, RwLock};
use chrono::Utc;

use std::io::BufReader;
use std::io::BufRead;
use embedded_io_adapters::std::ToStd;
use enumset::EnumSet;
use esp_idf_hal::{
    prelude::*,
    delay::TickType,
    gpio::AnyIOPin,
    uart, 
    uart::{UART1, config::*},
    //io::BufRead,
};

use nmea_parser::{
    NmeaParser,
    ParsedMessage,
};

#[named]
pub fn test_uart(
    pin_rx: AnyIOPin,
    pin_tx: AnyIOPin,
    uart_device: UART1,
    time_offset: Arc<RwLock<i64>>,
) ->  Result<()>  {
    info!(target: function_name!(), "Connecting to GPIO 17 to sample the sensor");

    std::thread::sleep(core::time::Duration::from_millis(500));

    // Initialize config manually, because `uart::config::Config::default()`
    // crashes on ESP32-C6 due to an invalid SourceClock.
    let config = uart::config::Config {
        baudrate: Hertz(9_600),
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
    )?;

    let mut cycles_without_data = 0;
    info!(target: function_name!(), "Try to read stuff...");
    let mut buf_reader: BufReader<_> = BufReader::new(ToStd::new(uart));
    let mut parser = NmeaParser::new();
    loop {
        let mut line = String::new();
        let len = buf_reader.read_line(&mut line).unwrap_or_default();
       
        if len >= 3 {
            cycles_without_data = 0;
            match parser.parse_sentence(line.as_str()) {
                Ok(ParsedMessage::Rmc(rmc)) => {
                    match rmc.timestamp {
                        Some(time_gps) => {
                            let timestamp_gps = time_gps.timestamp();
                            let now = Utc::now();
                            let timestamp_local = now.timestamp();
                            let offset = timestamp_gps - timestamp_local;
                            let mut time_offset_write = time_offset.write().unwrap();
                            *time_offset_write = offset;
        
                            info!(target: function_name!(), "Time from GPS: {}, Local time: {}, offset: {}", time_gps, now, offset);
                        },
                        _ => {}
                    }
                   
                }
                _ => {
                }
            }
            
        } else {
            cycles_without_data += 1;
            if cycles_without_data > 100 && cycles_without_data % 100 == 0 {
                warn!(target: function_name!(), "Had {} cycles without any data.", cycles_without_data);
            }
        }
    }
}
