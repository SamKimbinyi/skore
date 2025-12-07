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

    pub fn decode(&self, bytes: &[u8]) -> Result<(RespValue, usize), ParseError> {
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
        let string_end = find_crlf(bytes, 1)? + 1;
        let content = &bytes[1..string_end];

        let s = std::str::from_utf8(content)
            .map_err(|_| ParseError::InvalidUtf8)?
            .to_string();

        Ok((RespValue::SimpleString(s), string_end + 2))
    }

    fn decode_error(bytes: &[u8]) -> Result<(RespValue, usize), ParseError> {
        Self::decode_simple_string(bytes)
    }

    fn decode_integer(bytes: &[u8]) -> Result<(RespValue, usize), ParseError> {
        todo!()
    }

    fn decode_bulk_string(bytes: &[u8]) -> Result<(RespValue, usize), ParseError> {
        todo!()
    }

    fn decode_array(bytes: &[u8]) -> Result<(RespValue, usize), ParseError> {
        todo!()
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
}
