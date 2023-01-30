#![no_std]

use arraydeque::{ArrayDeque, Wrapping};

#[cfg(feature = "embedded-hal")]
use embedded_hal::serial::Read;

// Important bytes for correctnes checks
const FLAG_MASK: u8 = 0b11110000;
const HEAD_BYTE: u8 = 0b00001111;
const FOOT_BYTE: u8 = 0b00000000;

// Number of bytes in SBUS message
const PACKET_SIZE: usize = 25;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub struct SBusPacket {
    pub channels: [u16; 16],
    pub d1: bool,
    pub d2: bool,
    pub failsafe: bool,
    pub frame_lost: bool,
}

pub struct SBusPacketParser {
    buffer: ArrayDeque<[u8; (PACKET_SIZE * 2) as usize], Wrapping>,
}

impl SBusPacketParser {
    pub fn new() -> SBusPacketParser {
        SBusPacketParser {
            buffer: ArrayDeque::new(),
        }
    }

    /// Push single `u8` byte into buffer.
    #[inline(always)]
    pub fn push_byte(&mut self, byte: u8) {
        let _ = self.buffer.push_back(byte);
    }

    /// Push array of `u8` bytes into buffer.
    pub fn push_bytes(&mut self, bytes: &[u8]) {
        bytes.iter().for_each(|byte| {
            self.push_byte(*byte);
        })
    }

    /// Exhaustively reads the bytes from uart device implenting
    /// the `embedded_hal::serial::Read<u8>` trait.
    #[cfg(feature = "embedded-hal")]
    pub fn read_serial<U : Read<u8>>(&mut self, uart : & mut U) {
        while let Ok(byte) = uart.read() {
            self.push_byte(byte);
        }
    }

    /// Equivalent to consequtively calling `read_serial()` and `try_parse()`.
    #[cfg(feature = "embedded-hal")]
    pub fn read_serial_try_parse<U : Read<u8>>
    (&mut self, uart : & mut U) -> Option<SBusPacket> {
        self.read_serial(uart);
        self.try_parse()
    }

    /// Attempts to parse a valid SBUS packet from the buffer
    pub fn try_parse(&mut self) -> Option<SBusPacket> {
        
        // Pop bytes until head byte is first        
        while *self.buffer.front()? != HEAD_BYTE
        && self.buffer.len() > PACKET_SIZE {
            self.buffer.pop_front()?;
        }

        // Check if entire frame is valid
        if !self._valid_frame() { return None }

        // Extract the relevant data from buffer
        let mut data = [0; 24];
        for d in data.iter_mut() {
            *d = self.buffer.pop_front()? as u16
        }

        // Initialize channels with 11-bit mask
        let mut ch: [u16; 16] = [0x07FF; 16];

        // Trust me bro
        ch[0]  &= data[1]       | data[2]  << 8;
        ch[1]  &= data[2]  >> 3 | data[3]  << 5;
        ch[2]  &= data[3]  >> 6 | data[4]  << 2 | data[5] << 10;
        ch[3]  &= data[5]  >> 1 | data[6]  << 7;
        ch[4]  &= data[6]  >> 4 | data[7]  << 4;
        ch[5]  &= data[7]  >> 7 | data[8]  << 1 | data[9] << 9;
        ch[6]  &= data[9]  >> 2 | data[10] << 6;
        ch[7]  &= data[10] >> 5 | data[11] << 3;
        
        ch[8]  &= data[12]      | data[13] << 8;
        ch[9]  &= data[13] >> 3 | data[14] << 5;
        ch[10] &= data[14] >> 6 | data[15] << 2 | data[16] << 10;
        ch[11] &= data[16] >> 1 | data[17] << 7;
        ch[12] &= data[17] >> 4 | data[18] << 4;
        ch[13] &= data[18] >> 7 | data[19] << 1 | data[20] << 9;
        ch[14] &= data[20] >> 2 | data[21] << 6;
        ch[15] &= data[21] >> 5 | data[22] << 3;

        let flag_byte = *data.get(23)? as u8;

        return Some(SBusPacket {
            channels:   ch,
            d1:         is_flag_set(flag_byte, 0),
            d2:         is_flag_set(flag_byte, 1),
            frame_lost: is_flag_set(flag_byte, 2),
            failsafe:   is_flag_set(flag_byte, 3),
        });
    }

    /// Returns `true` if the first part of the buffer contains a valid SBUS frame
    fn _valid_frame (&self) -> bool {
        if let (Some(head),Some(foot),Some(flag)) =
        (self.buffer.front(),self.buffer.get(PACKET_SIZE - 1),self.buffer.get(PACKET_SIZE - 2)) {

            // If the header, footer, and flag bytes exist, this condition should hold true
            *head == HEAD_BYTE && *foot == FOOT_BYTE && *flag & FLAG_MASK == 0

        } else { false }
    }
}

#[inline(always)]
fn is_flag_set(flag_byte: u8, shift_by: u8) -> bool {
    ( flag_byte >> shift_by ) & 1 == 1
}
