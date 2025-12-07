use skore_protocol::RespValue;

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Get { key: Vec<u8> },
    Set { key: Vec<u8>, value: Vec<u8> },
    Delete { key: Vec<u8> },
    Ping,
    Keys,
    Clear,
    Len,
    Unknown(String), // Maybe an error?
}

fn extract_bulk_string(value: &RespValue) -> Result<Vec<u8>, String> {
    match value {
        RespValue::BulkString(None) => Err("Unexpected null bulk string".to_string()),
        RespValue::BulkString(Some(bytes)) => Ok(bytes.clone()),
        _ => Err("Unexpected response from bulk string".to_string()),
    }
}
impl Command {
    pub fn from_resp(value: RespValue) -> Result<Self, String> {
        match value {
            RespValue::Array(elements) if !elements.is_empty() => {
                let command_name = match &elements[0] {
                    RespValue::BulkString(Some(bytes)) => String::from_utf8(bytes.clone())
                        .map_err(|_| "Invalid UTF-8 in command".to_string())?
                        .to_uppercase(),
                    _ => return Err("First element must be a bulk string".to_string()),
                };

                match command_name.as_str() {
                    "GET" => {
                        if elements.len() != 2 {
                            return Err("GET requires exactly 1 argument".to_string());
                        }
                        let key = extract_bulk_string(&elements[1])?;
                        Ok(Command::Get { key })
                    }
                    "SET" => {
                        if elements.len() != 3 {
                            return Err("SET requires exactly 2 arguments".to_string());
                        }
                        let key = extract_bulk_string(&elements[1])?;
                        let value = extract_bulk_string(&elements[2])?;
                        Ok(Command::Set { key, value })
                    }
                    "DEL" | "DELETE" => {
                        if elements.len() != 2 {
                            return Err("DELETE requires exactly 1 argument".to_string());
                        }
                        let key = extract_bulk_string(&elements[1])?;
                        Ok(Command::Delete { key })
                    }
                    "PING" => Ok(Command::Ping),
                    "KEYS" => Ok(Command::Keys),
                    "CLEAR" => Ok(Command::Clear),
                    "LEN" => Ok(Command::Len),
                    _ => Ok(Command::Unknown(command_name)),
                }
            }
            _ => Err("Command must be an array".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_get() {
        let resp = RespValue::Array(vec![RespValue::bulk(b"GET"), RespValue::bulk(b"mykey")]);

        let cmd = Command::from_resp(resp).unwrap();
        assert_eq!(
            cmd,
            Command::Get {
                key: b"mykey".to_vec()
            }
        );
    }

    #[test]
    fn test_parse_set() {
        let resp = RespValue::Array(vec![
            RespValue::bulk(b"SET"),
            RespValue::bulk(b"mykey"),
            RespValue::bulk(b"myvalue"),
        ]);

        let cmd = Command::from_resp(resp).unwrap();
        assert_eq!(
            cmd,
            Command::Set {
                key: b"mykey".to_vec(),
                value: b"myvalue".to_vec()
            }
        );
    }
}
