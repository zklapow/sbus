use criterion::{black_box, Criterion};
use hex_literal::hex;

pub fn bench_parser(c: &mut Criterion) {

    let mut parser = sbus::SBusPacketParser::new();

    let bytes =
        hex!("00 0F E0 03 1F 58 C0 07 16 B0 80 05 2C 60 01 0B F8 C0 07 00 00 00 00 00 03 00");

    c.bench_function("parser", |b| b.iter(||{

        parser.push_bytes(&bytes);
        let msg = parser.try_parse().unwrap();
        
        black_box(msg);
    }));
}
