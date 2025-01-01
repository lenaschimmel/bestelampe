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

use nom::IResult;
use nom::number::complete::be_u16;
use nom::bytes::complete::tag;
use nom::multi::length_data;
use nom::error_position;
use nom::branch::alt;

#[derive(Debug)]
pub enum Proximity {
    None,
    Approaching,
    MovingAway,
}

#[derive(Debug)]
pub enum Presence {
    Unoccupied,
    Occupied,
}

#[derive(Debug)]
pub enum Motion {
    None,
    Motionless,
    Active,
}

#[derive(Debug)]
pub enum HumanPresence {
    PresenceInformation(Presence),
    MotionInformation(Motion),
    BodyMovementParameter(u8),
    ProximityReport(Proximity),
}

#[derive(Debug)]
pub enum Frame {
    Heartbeat,
    ProductInformation,
    UartUpgrade,
    OperationStatus,
    HumanPresenceReport(HumanPresence),
}

fn parse_proximity_information(input: &[u8]) -> IResult<&[u8], HumanPresence> {
    let (input, _) = tag([0x0b])(input)?;
    let (input, data) = length_data(be_u16)(input)?;

    let (_, data_byte) = nom::number::complete::u8(data)?;
    let proximity = match data_byte {
        0x00 => Proximity::None,
        0x01 => Proximity::Approaching,
        0x02 => Proximity::MovingAway,
        _ => return Err(nom::Err::Error(error_position!(input, nom::error::ErrorKind::IsNot))), // Help, I do not know how to return a custom error here!
    };
    return Ok((input, HumanPresence::ProximityReport(proximity)))
}

fn parse_presence_information(input: &[u8]) -> IResult<&[u8], HumanPresence> {
    let (input, _) = tag([0x01])(input)?;
    let (input, data) = length_data(be_u16)(input)?;

    let (_, data_byte) = nom::number::complete::u8(data)?;
    let presence = match data_byte {
        0x00 => Presence::Unoccupied,
        0x01 => Presence::Occupied,
        _ => return Err(nom::Err::Error(error_position!(input, nom::error::ErrorKind::Tag))), // Help, I do not know how to return a custom error here!
    };
    return Ok((input, HumanPresence::PresenceInformation(presence)))
}

fn parse_motion_information(input: &[u8]) -> IResult<&[u8], HumanPresence> {
    let (input, _) = tag([0x02])(input)?;
    let (input, data) = length_data(be_u16)(input)?;

    let (_, data_byte) = nom::number::complete::u8(data)?;
    let motion = match data_byte {
        0x00 => Motion::None,
        0x01 => Motion::Motionless,
        0x02 => Motion::Active,
        _ => return Err(nom::Err::Error(error_position!(input, nom::error::ErrorKind::Tag))), // Help, I do not know how to return a custom error here!
    };
    return Ok((input, HumanPresence::MotionInformation(motion)))
}

fn parse_body_movement_information(input: &[u8]) -> IResult<&[u8], HumanPresence> {
    let (input, _) = tag([0x03])(input)?;
    let (input, data) = length_data(be_u16)(input)?;
    let (_, data_byte) = nom::number::complete::u8(data)?;
        
    return Ok((input, HumanPresence::BodyMovementParameter(data_byte)))
}

fn parse_human_presence_report(input: &[u8]) -> IResult<&[u8], Frame> {
    let (input, _) = tag([0x80])(input)?;
        let (input, human_presence) = alt((
        parse_proximity_information,
        parse_presence_information,
        parse_motion_information,
        parse_body_movement_information,
    ))(input)?;
        Ok((input, Frame::HumanPresenceReport(human_presence)))
}

pub fn mr_parser(input: &[u8]) -> IResult<&[u8], Frame> {
    let (input, _) = tag([0x53, 0x59])(input)?;
    
    /*
    let (input, control) = nom::number::complete::u8(s)?;

    let frame = match control {
        0x01 => Frame::Heartbeat,
        0x02 => Frame::ProductInformation,
        0x03 => Frame::UartUpgrade,
        0x05 => Frame::OperationStatus,
        0x80 => {
            let (_, presence_report ) = parse_human_presence_report(input)?;
            Frame::HumanPresenceReport(presence_report)
        },
        _ => return Err(nom::Err::Error(error_position!(input, nom::error::ErrorKind::Tag))), // Help, I do not know how to return a custom error here!
    };
    */

    let (input, frame) = alt((
        parse_human_presence_report,
    ))(input)?;
    
    let (input, _checksum) = nom::number::complete::u8(input)?;
    let (input, _) = tag([0x54, 0x43])(input)?;

    Ok((input, frame))
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use super::*;

    const TEST_INPUT: [u8; 65] = hex!(
        "53 59 80 03 00 01 06 36 54 43 00 53 59 80 03 00 01 06 36 54 43 00"
        "53 59 80 03 00 01 06 36 54 43 00 00 53 59 80 03 00 01 0c 3c 54 43 00"
        "53 59 80 03 00 01 06 36 54 43 00 53 59 80 03 00 01 06 36 54"
    );

    #[test]
    fn test_mr_parser() {
        println!("Proximity: {:x?}", parse_proximity_information(&hex!("0b 00 01 02")).unwrap());
        println!("Human presence report: {:x?}", parse_human_presence_report(&hex!("80 0b 00 01 02")).unwrap());
        println!("Full frame: {:x?}", mr_parser(&hex!("53 59 80 03 00 01 06 36 54 43")).unwrap());
    }
}