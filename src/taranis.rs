use crate::SBusPacket;

const X7_MIN: u16 = 172;
const X7_MAX: u16 = 1811;
const X7_RANGE: f32 = (X7_MAX - X7_MIN) as f32;

#[derive(Debug, Copy, Clone)]
pub struct TaranisX7SBusPacket {
    channels: [f32; 16],
    failsafe: bool,
    frame_lost: bool,
}

impl TaranisX7SBusPacket {
    pub fn new(source: SBusPacket) -> TaranisX7SBusPacket {
        let mut mapped_channels: [f32; 16] = [0f32; 16];
        for i in 0..16 {
            mapped_channels[i] = TaranisX7SBusPacket::map(source.channels[i]);
        }

        return TaranisX7SBusPacket {
            channels: mapped_channels,
            failsafe: source.failsafe,
            frame_lost: source.frame_lost,
        };
    }

    fn map(v: u16) -> f32 {
        let offset = v - X7_MIN;

        return offset as f32 / X7_RANGE;
    }
}
