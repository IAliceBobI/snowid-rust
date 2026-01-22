#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_base62_encoding_decoding() {
        // Test basic encoding/decoding
        let test_values = [0u64, 1, 62, 123, 1234567890, u64::MAX / 2, u64::MAX];

        for &value in &test_values {
            let encoded = base62_encode(value);
            let decoded = base62_decode(&encoded).unwrap();
            assert_eq!(decoded, value, "Failed roundtrip for {value}");
        }
    }

    #[test]
    fn test_base62_generator_consistency() {
        // Create a generator with a custom config
        let config = SnowIDConfig::default();
        let generator = SnowID::with_config(42, config).unwrap();

        // Generate both regular and base62 IDs with the same generator
        let regular_id = generator.generate();
        let (base62_id, raw_id) = generator.generate_base62_with_raw();

        // Ensure the raw ID can be decoded from the string
        let decoded_id = base62_decode(&base62_id).unwrap();
        assert_eq!(decoded_id, raw_id);

        // Extract components from both IDs
        let (reg_ts, reg_node, reg_seq) = generator.extract.decompose(regular_id);
        let (base_ts, base_node, base_seq) = generator.decompose_base62(&base62_id).unwrap();

        // Verify node IDs are correct
        assert_eq!(reg_node, 42);
        assert_eq!(base_node, 42);

        // Timestamps should be reasonable
        assert!(reg_ts > 0);
        assert!(base_ts > 0);

        // Sequences should be within bounds
        assert!(reg_seq < config.max_sequence_id());
        assert!(base_seq < config.max_sequence_id());
    }

    #[test]
    fn test_base62_error_handling() {
        let generator = SnowID::new(1).unwrap();

        // Test invalid characters
        assert!(generator.decode_base62("abc!def").is_err());

        // Test empty string
        assert!(generator.decode_base62("").is_err());

        // Test decompose with invalid input
        assert!(generator.decompose_base62("invalid!").is_err());
    }

    #[test]
    fn test_base62_id_length() {
        let generator = SnowID::new(1).unwrap();

        // Generate multiple IDs and check their length
        for _ in 0..10 {
            let id = generator.generate_base62();

            // Base62 encoded snowids should be relatively short
            // For a 64-bit integer, the max length in base62 is 11 characters
            assert!(
                id.len() <= 11,
                "Base62 ID length should be <= 11, got {}",
                id.len()
            );

            // Ensure we can decode it back
            let decoded = generator.decode_base62(&id).unwrap();
            assert!(decoded > 0);
        }
    }

    #[test]
    fn test_base62_input_length_validation() {
        let generator = SnowID::new(1).unwrap();

        // Test that input longer than maximum u64 in base62 is rejected
        let long_input = "a".repeat(12); // 12 characters > MAX_BASE62_LEN (11)
        let result = generator.decode_base62(&long_input);
        assert!(
            result.is_err(),
            "Should reject input longer than 11 characters"
        );

        // Verify the error type is correct
        match result {
            Err(Base62DecodeError::InvalidInput) => {}
            _ => panic!("Expected InvalidInput error for long input"),
        }

        // Test that 11 character input is accepted (if valid base62)
        let max_valid_input = "4Ly3K1aP0d0"; // u64::MAX in base62
        let result = generator.decode_base62(max_valid_input);
        // This should not fail due to length (may fail for other reasons if input is invalid)
        if let Err(Base62DecodeError::InvalidInput) = result {
            panic!("Should accept 11 character input");
        }
    }
}
