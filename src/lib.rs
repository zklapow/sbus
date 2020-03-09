#![no_std]

mod taranis;

pub use taranis::TaranisX7SBusPacket;

use arraydeque::{ArrayDeque, Wrapping};

// The flag by should start with 4 0s
const SBUS_FLAG_BYTE_MASK: u8 = 0b11110000;
const SBUS_HEADER_BYTE: u8 = 0x0F;
const SBUS_FOOTER_BYTE: u8 = 0b00000000;

const SBUS_PACKET_SIZE: usize = 25;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub struct SBusPacket {
    channels: [u16; 16],
    d1: bool,
    d2: bool,
    failsafe: bool,
    frame_lost: bool,
}

pub struct SBusPacketParser {
    buffer: ArrayDeque<[u8; (SBUS_PACKET_SIZE * 2) as usize], Wrapping>,
}

impl SBusPacketParser {
    pub fn new() -> SBusPacketParser {
        SBusPacketParser {
            buffer: ArrayDeque::new(),
        }
    }

    pub fn push_bytes(&mut self, bytes: &[u8]) {
        bytes.iter().for_each(|b| {
            self.buffer.push_back(*b);
        })
    }

    pub fn try_parse(&mut self) -> Option<SBusPacket> {
        // We can't have a packet if we don't have enough bytes
        if self.buffer.len() < SBUS_PACKET_SIZE {
            return None;
        }

        // If the first byte is not a header byte,
        if *self.buffer.get(0).unwrap() != SBUS_HEADER_BYTE {
            while self.buffer.len() > 0 && *self.buffer.get(0).unwrap() != SBUS_HEADER_BYTE {
                let _ = self.buffer.pop_front().unwrap();
                //println!("Popped byte: {:#?}", popped);
            }

            return None;
        } else if *self.buffer.get(SBUS_PACKET_SIZE - 1).unwrap() == SBUS_FOOTER_BYTE
            && self.buffer.get(SBUS_PACKET_SIZE - 2).unwrap() & SBUS_FLAG_BYTE_MASK == 0
        {
            // This seems like a valid packet!
            // Start popping the bytes

            let mut data_bytes: [u16; 23] = [0; 23];
            for i in 0..23 {
                data_bytes[i] = self.buffer.pop_front().unwrap_or(0) as u16;
            }

            let mut channels: [u16; 16] = [0; 16];

            channels[0] = (((data_bytes[1]) | (data_bytes[2] << 8)) as u16 & 0x07FF).into();
            channels[1] = ((((data_bytes[2] >> 3) | (data_bytes[3] << 5)) as u16) & 0x07FF).into();
            channels[2] = ((((data_bytes[3] >> 6) | (data_bytes[4] << 2) | (data_bytes[5] << 10))
                as u16)
                & 0x07FF)
                .into();
            channels[3] = ((((data_bytes[5] >> 1) | (data_bytes[6] << 7)) as u16) & 0x07FF).into();
            channels[4] = ((((data_bytes[6] >> 4) | (data_bytes[7] << 4)) as u16) & 0x07FF).into();
            channels[5] = ((((data_bytes[7] >> 7) | (data_bytes[8] << 1) | (data_bytes[9] << 9))
                as u16)
                & 0x07FF)
                .into();
            channels[6] = ((((data_bytes[9] >> 2) | (data_bytes[10] << 6)) as u16) & 0x07FF).into();
            channels[7] =
                ((((data_bytes[10] >> 5) | (data_bytes[11] << 3)) as u16) & 0x07FF).into();
            channels[8] = ((((data_bytes[12]) | (data_bytes[13] << 8)) as u16) & 0x07FF).into();
            channels[9] =
                ((((data_bytes[13] >> 3) | (data_bytes[14] << 5)) as u16) & 0x07FF).into();
            channels[10] =
                ((((data_bytes[14] >> 6) | (data_bytes[15] << 2) | (data_bytes[16] << 10)) as u16)
                    & 0x07FF)
                    .into();
            channels[11] =
                ((((data_bytes[16] >> 1) | (data_bytes[17] << 7)) as u16) & 0x07FF).into();
            channels[12] =
                ((((data_bytes[17] >> 4) | (data_bytes[18] << 4)) as u16) & 0x07FF).into();
            channels[13] =
                ((((data_bytes[18] >> 7) | (data_bytes[19] << 1) | (data_bytes[20] << 9)) as u16)
                    & 0x07FF)
                    .into();
            channels[14] =
                ((((data_bytes[20] >> 2) | (data_bytes[21] << 6)) as u16) & 0x07FF).into();
            channels[15] =
                ((((data_bytes[21] >> 5) | (data_bytes[22] << 3)) as u16) & 0x07FF).into();

            let flag_byte = self.buffer.pop_front().unwrap_or(0);

            return Some(SBusPacket {
                channels,
                d1: is_flag_set(flag_byte, 0),
                d2: is_flag_set(flag_byte, 1),
                frame_lost: is_flag_set(flag_byte, 2),
                failsafe: is_flag_set(flag_byte, 3),
            });
        } else {
            // We had a header byte, but this doesnt appear to be a valid frame, we are probably out of sync
            // Pop until we find a header again
            while self.buffer.len() > 0 && *self.buffer.get(0).unwrap() != SBUS_HEADER_BYTE {
                self.buffer.pop_front();
            }
        }

        return None;
    }
}

fn is_flag_set(flag_byte: u8, idx: u8) -> bool {
    flag_byte & 1 << idx == 1
}
