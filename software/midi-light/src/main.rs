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
//
// -----
//
// This is a simple command line program that connects to a MIDI input device and via serial 
// UART to a microcontroller that runs the tlc59711-programm. I use it with a Arturia Minilab
// and it needs a specific configuration of the knobs to work, which is not yet documented.

use std::error::Error;
use std::io::{stdin, stdout, Write};
use std::env;
use std::thread;
use serial::prelude::*;
use std::time::Duration;

use midir::{Ignore, MidiInput};

fn main() {
    match run() {
        Ok(_) => (),
        Err(err) => println!("Error: {}", err),
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut input = String::new();

    let mut midi_in = MidiInput::new("midir reading input")?;
    midi_in.ignore(Ignore::None);

    // Get an input port (read from console if multiple are available)
    let in_ports = midi_in.ports();
    let in_port = match in_ports.len() {
        0 => return Err("no input port found".into()),
        1 => {
            println!(
                "Choosing the only available input port: {}",
                midi_in.port_name(&in_ports[0]).unwrap()
            );
            &in_ports[0]
        }
        _ => {
            println!("\nAvailable input ports:");
            for (i, p) in in_ports.iter().enumerate() {
                println!("{}: {}", i, midi_in.port_name(p).unwrap());
            }
            print!("Please select input port: ");
            stdout().flush()?;
            let mut input = String::new();
            stdin().read_line(&mut input)?;
            in_ports
                .get(input.trim().parse::<usize>()?)
                .ok_or("invalid input port selected")?
        }
    };

    let port_name = env::args_os().nth(1).unwrap();
    println!("Try to open TTY: {:?}", port_name);
    let mut port = serial::open(&port_name).unwrap();
    port.reconfigure(&|settings| {
        settings.set_baud_rate(serial::Baud115200)?;
        settings.set_char_size(serial::Bits8);
        settings.set_parity(serial::ParityNone);
        settings.set_stop_bits(serial::Stop1);
        settings.set_flow_control(serial::FlowSoftware);
        Ok(())
    })?;

    port.set_timeout(Duration::from_millis(1000))?;
    println!("TTY is open anc conifgured.");

    println!("Opening MIDI connection");
    let in_port_name = midi_in.port_name(in_port)?;

    let mut values: [i16; 12] = [0; 12];
    let mut last_stamp = 0;

    // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
    let _conn_in = midi_in.connect(
        in_port,
        "midir-read-input",
        move |stamp, message, _| {
            if message[1] >= 20 && message[1] < 32 {
                let channel = (message[1] - 20) as usize;
                let delta = message[2] as i8 - 64;
                values[channel] = (values[channel] + (delta as i16)).clamp(0, 1024);
                
                print!("{:10}:\t\t", stamp);
                for c in 0 .. 12 {
                    print!("{:4}\t", values[c]);
                }
                print!("\n");
                
                if stamp > last_stamp + 30000 {
                    let mut s = Vec::new();
                    const SIZE: usize = 16;
                    write!(&mut s, "{}:{}\n", channel, values[channel]);
                    port.write(&s);
                    last_stamp = stamp;
                }
            }
        },
        (),
    )?;

    println!(
        "Connection open, reading input from '{}' (press enter to exit) ...",
        in_port_name
    );

    input.clear();
    stdin().read_line(&mut input)?; // wait for next enter key press

    println!("Closing connection");
    Ok(())
}