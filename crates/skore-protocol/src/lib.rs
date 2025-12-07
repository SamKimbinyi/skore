#[derive(Debug, Clone, PartialEq)]
pub enum RespValue {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Option<Vec<u8>>),
    Array(Vec<RespValue>),
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ParseError {
    #[error("Incomplete Data")]
    Incomplete,

    #[error("Invalid format {0}")]
    InvalidFormat(String),

    #[error("Invalid UTF-8")]
    InvalidUtf8,

    #[error("Invalid Integer")]
    InvalidInteger,
}

const CRLF: &[u8] = b"\r\n";

fn find_crlf(bytes: &[u8], start: usize) -> Result<usize, ParseError> {
    for i in start..bytes.len().saturating_sub(1) {
        if &bytes[i..i + 2] == CRLF {
            return Ok(i);
        }
    }
    Err(ParseError::Incomplete)
}

impl RespValue {
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        self.encode_to(&mut buf);

        buf
    }

    fn encode_to(&self, buf: &mut Vec<u8>) {
        match self {
            RespValue::SimpleString(s) => {
                buf.push(b'+');
                buf.extend_from_slice(s.as_bytes());
                buf.extend_from_slice(CRLF);
            }
            RespValue::Error(msg) => {
                buf.push(b'-');
                buf.extend_from_slice(msg.as_bytes());
                buf.extend_from_slice(CRLF);
            }
            RespValue::Integer(i) => {
                buf.push(b':');
                buf.extend_from_slice(i.to_string().as_bytes());
                buf.extend_from_slice(CRLF);
            }
            RespValue::BulkString(data) => match data {
                Some(bytes) => {
                    buf.push(b'$');
                    buf.extend_from_slice(format!("{}", bytes.len()).as_bytes());
                    buf.extend_from_slice(CRLF);
                    buf.extend_from_slice(bytes);
                    buf.extend_from_slice(CRLF);
                }
                None => {
                    buf.extend_from_slice(b"$-1\r\n");
                }
            },
            RespValue::Array(elements) => {
                buf.push(b'*');
                buf.extend_from_slice(format!("{}", elements.len()).as_bytes());
                buf.extend_from_slice(CRLF);

                for elem in elements {
                    elem.encode_to(buf);
                }
            }
        }
    }

    pub fn decode(bytes: &[u8]) -> Result<(RespValue, usize), ParseError> {
        if bytes.is_empty() {
            return Err(ParseError::Incomplete);
        }

        match bytes[0] {
            b'+' => Self::decode_simple_string(bytes),
            b'-' => Self::decode_error(bytes),
            b':' => Self::decode_integer(bytes),
            b'$' => Self::decode_bulk_string(bytes),
            b'*' => Self::decode_array(bytes),
            _ => Err(ParseError::InvalidFormat(format!(
                "Invalid type bytes{0}",
                bytes[0] as char
            ))),
        }
    }
    fn decode_simple_string(bytes: &[u8]) -> Result<(RespValue, usize), ParseError> {
        let line_end = find_crlf(bytes, 1)?;
        let content = &bytes[1..line_end];

        let s = std::str::from_utf8(content)
            .map_err(|_| ParseError::InvalidUtf8)?
            .to_string();

        Ok((RespValue::SimpleString(s), line_end + 2))
    }

    fn decode_error(bytes: &[u8]) -> Result<(RespValue, usize), ParseError> {
        let line_end = find_crlf(bytes, 1)?;
        let content = &bytes[1..line_end];

        let s = std::str::from_utf8(content)
            .map_err(|_| ParseError::InvalidUtf8)?
            .to_string();

        Ok((RespValue::Error(s), line_end + 2))
    }

    fn decode_integer(bytes: &[u8]) -> Result<(RespValue, usize), ParseError> {
        let line_end = find_crlf(bytes, 1)?;
        let content = &bytes[1..line_end];

        let s = std::str::from_utf8(content).map_err(|_| ParseError::InvalidUtf8)?;
        let int = s.parse::<i64>().map_err(|_| ParseError::InvalidInteger)?;
        Ok((RespValue::Integer(int), line_end + 2))
    }

    fn decode_bulk_string(bytes: &[u8]) -> Result<(RespValue, usize), ParseError> {
        let line_end = find_crlf(bytes, 1)?;
        let content = &bytes[1..line_end];

        let length_string = std::str::from_utf8(content).map_err(|_| ParseError::InvalidUtf8)?;
        let length = length_string
            .parse::<i64>()
            .map_err(|_| ParseError::InvalidInteger)?;

        if length == -1 {
            return Ok((RespValue::BulkString(None), line_end + 2));
        }

        if length < 0 {
            return Err(ParseError::InvalidFormat(
                "Bulk String length cannot be negative (except -1)".to_string(),
            ));
        }

        let length = length as usize;
        let data_start = line_end + 2;
        let data_end = data_start + length;

        if bytes.len() < data_end + 2 {
            return Err(ParseError::Incomplete);
        }

        if &bytes[data_end..data_end + 2] != CRLF {
            return Err(ParseError::InvalidFormat(
                "Missing CRLF after bulk string".to_string(),
            ));
        }

        let data = bytes[data_start..data_end].to_vec();
        Ok((RespValue::BulkString(Some(data)), data_end + 2))
    }

    fn decode_array(bytes: &[u8]) -> Result<(RespValue, usize), ParseError> {
        let line_end = find_crlf(bytes, 1)?;
        let count_string = &bytes[1..line_end];

        let count_string =
            std::str::from_utf8(count_string).map_err(|_| ParseError::InvalidUtf8)?;
        let count = count_string
            .parse::<usize>()
            .map_err(|_| ParseError::InvalidInteger)?;

        let mut elements = Vec::with_capacity(count);
        let mut pos = line_end + 2;

        for _ in 0..count {
            if pos >= bytes.len() {
                return Err(ParseError::Incomplete);
            }

            let (value, consumed) = Self::decode(&bytes[pos..])?;
            elements.push(value);
            pos += consumed;
        }

        Ok((RespValue::Array(elements), pos))
    }
    pub fn ok() -> Self {
        RespValue::SimpleString("OK".to_string())
    }

    pub fn error<S: Into<String>>(msg: S) -> Self {
        RespValue::Error(msg.into())
    }

    pub fn bulk<B: Into<Vec<u8>>>(data: B) -> Self {
        RespValue::BulkString(Some(data.into()))
    }

    pub fn null() -> Self {
        RespValue::BulkString(None)
    }

    pub fn array(values: Vec<Self>) -> Self {
        RespValue::Array(values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_crlf_after_start_offset() {
        let bytes = b"aaa\r\nbbb";
        let pos = find_crlf(bytes, 2).unwrap();
        assert_eq!(pos, 3);
    }

    #[test]
    fn returns_incomplete_when_missing() {
        let bytes = b"helloworld";
        let err = find_crlf(bytes, 0).unwrap_err();
        assert_eq!(err, ParseError::Incomplete);
    }

    #[test]
    fn test_resp_value_creation() {
        // Simple string
        let simple = RespValue::SimpleString("OK".to_string());
        assert_eq!(simple, RespValue::SimpleString("OK".to_string()));

        // Error
        let error = RespValue::Error("ERR unknown".to_string());
        assert_eq!(error, RespValue::Error("ERR unknown".to_string()));

        // Integer
        let int = RespValue::Integer(42);
        assert_eq!(int, RespValue::Integer(42));

        // Bulk string with data
        let bulk = RespValue::BulkString(Some(b"hello".to_vec()));
        assert_eq!(bulk, RespValue::BulkString(Some(b"hello".to_vec())));

        // Bulk string null
        let null = RespValue::BulkString(None);
        assert_eq!(null, RespValue::BulkString(None));

        // Array
        let array = RespValue::Array(vec![
            RespValue::BulkString(Some(b"GET".to_vec())),
            RespValue::BulkString(Some(b"key".to_vec())),
        ]);
        assert!(matches!(array, RespValue::Array(_)));
    }

    // ===== ENCODING TESTS =====

    #[test]
    fn test_encode_simple_string() {
        let value = RespValue::SimpleString("OK".to_string());
        let encoded = {
            let this = &value;
            let mut buf = Vec::new();
            this.encode_to(&mut buf);

            buf
        };
        assert_eq!(encoded, b"+OK\r\n");
    }

    #[test]
    fn test_encode_error() {
        let value = RespValue::Error("ERR unknown command".to_string());
        let encoded = {
            let this = &value;
            let mut buf = Vec::new();
            this.encode_to(&mut buf);

            buf
        };
        assert_eq!(encoded, b"-ERR unknown command\r\n");
    }

    #[test]
    fn test_encode_integer() {
        let value = RespValue::Integer(1000);
        assert_eq!(value.encode(), b":1000\r\n");

        let negative = RespValue::Integer(-42);
        assert_eq!(negative.encode(), b":-42\r\n");

        let zero = RespValue::Integer(0);
        assert_eq!(zero.encode(), b":0\r\n");
    }

    #[test]
    fn test_encode_bulk_string() {
        // Normal bulk string
        let value = RespValue::BulkString(Some(b"foobar".to_vec()));
        assert_eq!(value.encode(), b"$6\r\nfoobar\r\n");

        // Empty bulk string
        let empty = RespValue::BulkString(Some(Vec::new()));
        assert_eq!(empty.encode(), b"$0\r\n\r\n");

        // Null bulk string
        let null = RespValue::BulkString(None);
        assert_eq!(null.encode(), b"$-1\r\n");
    }

    #[test]
    fn test_encode_array() {
        // Empty array
        let empty = RespValue::Array(Vec::new());
        assert_eq!(empty.encode(), b"*0\r\n");

        // Array with two bulk strings: ["foo", "bar"]
        let array = RespValue::Array(vec![
            RespValue::BulkString(Some(b"foo".to_vec())),
            RespValue::BulkString(Some(b"bar".to_vec())),
        ]);
        assert_eq!(array.encode(), b"*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n");

        // Mixed types array
        let mixed = RespValue::Array(vec![
            RespValue::SimpleString("OK".to_string()),
            RespValue::Integer(42),
            RespValue::BulkString(Some(b"data".to_vec())),
        ]);
        assert_eq!(mixed.encode(), b"*3\r\n+OK\r\n:42\r\n$4\r\ndata\r\n");
    }

    #[test]
    fn test_encode_nested_array() {
        // Array containing an array: [["foo", "bar"], "baz"]
        let nested = RespValue::Array(vec![
            RespValue::Array(vec![
                RespValue::BulkString(Some(b"foo".to_vec())),
                RespValue::BulkString(Some(b"bar".to_vec())),
            ]),
            RespValue::BulkString(Some(b"baz".to_vec())),
        ]);

        assert_eq!(
            nested.encode(),
            b"*2\r\n*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n$3\r\nbaz\r\n"
        );
    }

    #[test]
    fn test_helper_constructors() {
        assert_eq!(RespValue::ok().encode(), b"+OK\r\n");

        assert_eq!(RespValue::error("test").encode(), b"-test\r\n");

        assert_eq!(
            RespValue::bulk(b"hello".to_vec()).encode(),
            b"$5\r\nhello\r\n"
        );

        assert_eq!(RespValue::null().encode(), b"$-1\r\n");
    }

    // ===== DECODING TESTS =====

    #[test]
    fn test_decode_simple_string() {
        let bytes = b"+OK\r\n";
        let (value, consumed) = RespValue::decode(bytes).unwrap();

        assert_eq!(value, RespValue::SimpleString("OK".to_string()));
        assert_eq!(consumed, 5);
    }

    #[test]
    fn test_decode_error() {
        let bytes = b"-ERR unknown command\r\n";
        let (value, consumed) = RespValue::decode(bytes).unwrap();

        assert_eq!(value, RespValue::Error("ERR unknown command".to_string()));
        assert_eq!(consumed, 22);
    }

    #[test]
    fn test_decode_integer() {
        let bytes = b":1000\r\n";
        let (value, consumed) = RespValue::decode(bytes).unwrap();

        assert_eq!(value, RespValue::Integer(1000));
        assert_eq!(consumed, 7);

        // Negative
        let bytes = b":-42\r\n";
        let (value, consumed) = RespValue::decode(bytes).unwrap();
        assert_eq!(value, RespValue::Integer(-42));
        assert_eq!(consumed, 6);
    }

    #[test]
    fn test_decode_bulk_string() {
        let bytes = b"$6\r\nfoobar\r\n";
        let (value, consumed) = RespValue::decode(bytes).unwrap();

        assert_eq!(value, RespValue::BulkString(Some(b"foobar".to_vec())));
        assert_eq!(consumed, 12);

        // Empty bulk string
        let bytes = b"$0\r\n\r\n";
        let (value, consumed) = RespValue::decode(bytes).unwrap();
        assert_eq!(value, RespValue::BulkString(Some(Vec::new())));
        assert_eq!(consumed, 6);

        // Null bulk string
        let bytes = b"$-1\r\n";
        let (value, consumed) = RespValue::decode(bytes).unwrap();
        assert_eq!(value, RespValue::BulkString(None));
        assert_eq!(consumed, 5);
    }

    #[test]
    fn test_decode_array() {
        // Empty array
        let bytes = b"*0\r\n";
        let (value, consumed) = RespValue::decode(bytes).unwrap();
        assert_eq!(value, RespValue::Array(Vec::new()));
        assert_eq!(consumed, 4);

        // Simple array
        let bytes = b"*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n";
        let (value, consumed) = RespValue::decode(bytes).unwrap();

        let expected = RespValue::Array(vec![
            RespValue::BulkString(Some(b"foo".to_vec())),
            RespValue::BulkString(Some(b"bar".to_vec())),
        ]);
        assert_eq!(value, expected);
        assert_eq!(consumed, 22);

        // Mixed types
        let bytes = b"*3\r\n+OK\r\n:42\r\n$4\r\ndata\r\n";
        let (value, consumed) = RespValue::decode(bytes).unwrap();

        let expected = RespValue::Array(vec![
            RespValue::SimpleString("OK".to_string()),
            RespValue::Integer(42),
            RespValue::BulkString(Some(b"data".to_vec())),
        ]);
        assert_eq!(value, expected);
        assert_eq!(consumed, 24);
    }

    #[test]
    fn test_decode_nested_array() {
        let bytes = b"*2\r\n*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n$3\r\nbaz\r\n";
        let (value, consumed) = RespValue::decode(bytes).unwrap();

        let expected = RespValue::Array(vec![
            RespValue::Array(vec![
                RespValue::BulkString(Some(b"foo".to_vec())),
                RespValue::BulkString(Some(b"bar".to_vec())),
            ]),
            RespValue::BulkString(Some(b"baz".to_vec())),
        ]);

        assert_eq!(value, expected);
        assert_eq!(consumed, 38);
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let test_cases = vec![
            RespValue::SimpleString("OK".to_string()),
            RespValue::Error("ERR".to_string()),
            RespValue::Integer(42),
            RespValue::Integer(-100),
            RespValue::BulkString(Some(b"hello world".to_vec())),
            RespValue::BulkString(None),
            RespValue::Array(vec![
                RespValue::SimpleString("GET".to_string()),
                RespValue::BulkString(Some(b"key".to_vec())),
            ]),
        ];

        for original in test_cases {
            let encoded = original.encode();
            let (decoded, consumed) = RespValue::decode(&encoded).unwrap();

            assert_eq!(original, decoded);
            assert_eq!(consumed, encoded.len());
        }
    }

    #[test]
    fn test_decode_incomplete() {
        // Missing CRLF
        let bytes = b"+OK";
        assert!(matches!(
            RespValue::decode(bytes),
            Err(ParseError::Incomplete)
        ));

        // Incomplete bulk string
        let bytes = b"$6\r\nfoo";
        assert!(matches!(
            RespValue::decode(bytes),
            Err(ParseError::Incomplete)
        ));

        // Incomplete array
        let bytes = b"*2\r\n+OK\r\n";
        assert!(matches!(
            RespValue::decode(bytes),
            Err(ParseError::Incomplete)
        ));
    }

    #[test]
    fn test_decode_invalid_format() {
        // Unknown type byte
        let bytes = b"@invalid\r\n";
        assert!(matches!(
            RespValue::decode(bytes),
            Err(ParseError::InvalidFormat(_))
        ));

        // Invalid integer
        let bytes = b":not_a_number\r\n";
        assert!(matches!(
            RespValue::decode(bytes),
            Err(ParseError::InvalidInteger)
        ));
    }

    #[test]
    fn test_decode_multiple_values() {
        // Test parsing multiple values from the same buffer
        let bytes = b"+OK\r\n+HELLO\r\n:42\r\n";

        let (val1, consumed1) = RespValue::decode(bytes).unwrap();
        assert_eq!(val1, RespValue::SimpleString("OK".to_string()));
        assert_eq!(consumed1, 5);

        let (val2, consumed2) = RespValue::decode(&bytes[consumed1..]).unwrap();
        assert_eq!(val2, RespValue::SimpleString("HELLO".to_string()));
        assert_eq!(consumed2, 8);

        let (val3, consumed3) = RespValue::decode(&bytes[consumed1 + consumed2..]).unwrap();
        assert_eq!(val3, RespValue::Integer(42));
        assert_eq!(consumed3, 5);
    }

    #[test]
    fn test_bulk_string_with_binary_data() {
        // Test bulk string containing non-UTF8 data and special characters
        let binary_data = vec![0xFF, 0xFE, 0x00, b'\r', b'\n', 0x42];
        let value = RespValue::BulkString(Some(binary_data.clone()));

        let encoded = value.encode();
        let (decoded, _) = RespValue::decode(&encoded).unwrap();

        if let RespValue::BulkString(Some(data)) = decoded {
            assert_eq!(data, binary_data);
        } else {
            panic!("Expected BulkString");
        }
    }
}
