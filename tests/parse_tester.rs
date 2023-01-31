#[cfg(test)]
mod tests {
    use hex_literal::hex;

    const RAW_BYTES : [u8;25] =
        hex!("0F E0 03 1F 58 C0 07 16 B0 80 05 2C 60 01 0B F8 C0 07 00 00 00 00 00 03 00");

    const DECODED_PACKET : sbus::SBusPacket = sbus::SBusPacket {
        channels: [992, 992, 352, 992, 352, 352, 352, 352, 352, 352, 992, 992, 0, 0, 0, 0],
        d1: true,
        d2: true,
        failsafe: false,
        frame_lost: false
    };


    #[test]
    fn test_sbus_parse_basic() {

        let mut parser = sbus::SBusPacketParser::new();

        // Push full message
        parser.push_bytes(&RAW_BYTES);

        assert_eq!( Some( DECODED_PACKET ), parser.try_parse() );
    }

    #[test]
    fn test_sbus_parse_with_shifting() {

        let mut parser = sbus::SBusPacketParser::new();
        
        // Push message within other garbage bytes
        parser.push_bytes(&RAW_BYTES[5..15]);
        parser.push_bytes(&RAW_BYTES); // Actual message
        parser.push_bytes(&RAW_BYTES[5..15]);

        assert_eq!( Some( DECODED_PACKET ), parser.try_parse() );
    }

    #[test]
    fn test_sbus_error_missing_byte() {

        let mut parser = sbus::SBusPacketParser::new();
        
        // Push bytes with one missing (and some extra bytes)
        parser.push_bytes(&RAW_BYTES[0..7]);
        parser.push_bytes(&RAW_BYTES[8..25]);
        parser.push_bytes(&RAW_BYTES[8..10]);

        assert_eq!( None, parser.try_parse() );
    }

    #[test]
    fn test_sbus_error_too_short() {

        let mut parser = sbus::SBusPacketParser::new();
        
        // Push message that is too short
        parser.push_bytes(&RAW_BYTES[0..23]);
        
        assert_eq!( None, parser.try_parse() );
    }

    #[test]
    fn test_sbus_parse_two_parter() {

        let mut parser = sbus::SBusPacketParser::new();
        
        // Push first bytes of message
        parser.push_bytes(&RAW_BYTES[0..10]);

        assert_eq!( None, parser.try_parse() );

        // Push remaining message
        parser.push_bytes(&RAW_BYTES[10..25]);

        assert_eq!( Some(DECODED_PACKET), parser.try_parse() );
    }
}
