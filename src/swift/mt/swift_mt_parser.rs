use crate::swift::mt::model::{ApplicationHeader, BasicHeader, Trailer, UserHeader};
use crate::utils::StringParser;
use std::collections::HashMap;
use std::io;

pub struct SwiftMtParser {}

#[derive(Debug)]
pub struct ParsingError {
    pub message: String,
}

#[derive(Debug)]
pub struct Block {
    content: String,
}

pub struct SwiftMtMessage {
    pub application_header: ApplicationHeader,
    pub basic_header: BasicHeader,
    pub user_header: UserHeader,
    pub trailer: Trailer,
}

impl SwiftMtParser {
    pub fn new() -> SwiftMtParser {
        SwiftMtParser {}
    }

    pub fn parse(self, msg: String) -> Result<SwiftMtMessage, ParsingError> {
        let blocks = self.parse_blocks(msg)?;

        let bh = blocks
            .get(&'1')
            .map(|block| read_basic_header(block))
            .unwrap_or_else(|| Ok(BasicHeader::new()))?;
        let ah = blocks
            .get(&'2')
            .map(|block| read_application_header(block))
            .unwrap_or_else(|| Ok(ApplicationHeader::Empty))?;
        let uh = blocks
            .get(&'3')
            .map(|block| read_user_header(block))
            .unwrap_or_else(|| Ok(UserHeader::new()))?;
        let tr = blocks
            .get(&'5')
            .map(|block| read_trailer(block))
            .unwrap_or_else(|| Ok(Trailer::new()))?;

        let ret_msg = SwiftMtMessage {
            application_header: ah,
            basic_header: bh,
            user_header: uh,
            trailer: tr,
        };

        return Ok(ret_msg);
    }

    fn parse_blocks(self, msg: String) -> Result<HashMap<char, Block>, ParsingError> {
        let parser = StringParser::new(msg);
        return read_blocks(parser);
    }
}

static VALID_BLOCKS: [char; 6] = ['1', '2', '3', '4', '5', 'S'];

fn read_application_header(block: &Block) -> Result<ApplicationHeader, ParsingError> {
    return ApplicationHeader::from_raw(&mut StringParser::new(block.content.clone())).map_err(
        |e: io::Error| ParsingError {
            message: format!("Error reading application header: {:?}", e),
        },
    );
}

fn read_basic_header(block: &Block) -> Result<BasicHeader, ParsingError> {
    return BasicHeader::from_raw(&mut StringParser::new(block.content.clone())).map_err(
        |e: io::Error| ParsingError {
            message: format!("Error reading basic header: {:?}", e),
        },
    );
}

fn read_user_header(block: &Block) -> Result<UserHeader, ParsingError> {
    return UserHeader::from_raw(block.content.clone()).map_err(|e: io::Error| ParsingError {
        message: format!("Error reading user header: {:?}", e),
    });
}

fn read_trailer(block: &Block) -> Result<Trailer, ParsingError> {
    return Trailer::from_raw(block.content.clone()).map_err(|e: io::Error| ParsingError {
        message: format!("Error reading user header: {:?}", e),
    });
}

fn read_blocks(mut parser: StringParser) -> Result<HashMap<char, Block>, ParsingError> {
    let mut blocks: HashMap<char, Block> = HashMap::new();

    loop {
        if !parser.has_more() {
            break;
        }

        let mut start = parser.next();
        if start.as_ref().ok() != Some(&'{') {
            return Err(ParsingError {
                message: format!(
                    "Invalid message format, expected start of block ({{) but got {}",
                    start.map_or("end of stream".into(), |c| { c.to_string() })
                )
                .to_string(),
            });
        }

        start = parser.next();
        if start.is_err() {
            return Err(ParsingError {
                message: "Invalid message format, expected digit but got end of stream".to_string(),
            });
        }

        let separator = parser.next();
        if separator.as_ref().ok() != Some(&':') {
            return Err(ParsingError {
                message: format!(
                    "Invalid message format, expected separator (:) but got {}",
                    separator.map_or("end of stream".into(), |c| { c.to_string() })
                )
                .to_string(),
            });
        }

        let block_type = start.unwrap();
        if !VALID_BLOCKS.contains(&block_type) || block_type == '1' || block_type == '2' {
            let content = parser.until('}').unwrap();
            blocks.insert(block_type, Block { content });
        } else if block_type == '3' || block_type == '5' || block_type == 'S' {
            blocks.insert(block_type, read_system_block(&mut parser)?);
        } else if block_type == '4' {
            blocks.insert(block_type, read_message_text(&mut parser)?);
        }
    }

    return Ok(blocks);
}

fn read_message_text(parser: &mut StringParser) -> Result<Block, ParsingError> {
    let mut content = String::new();
    parser.next_line().map_err(|_e| {
        return ParsingError {
            message: "Invalid message format, reached end of stream while reading block 4"
                .to_string(),
        };
    })?; // skip over the first newline after {4:
    loop {
        if !parser.has_more() {
            return Err(ParsingError { message: "Invalid message format, got end of stream while reading block 4 before reading -}".to_string() });
        }

        let pos = parser.position();
        let line = parser.next_line().map_err(|_e| {
            return ParsingError {
                message: "Invalid message format, reached end of stream while reading block 4"
                    .to_string(),
            };
        })?;
        if line.starts_with("-}") {
            parser.set_position(pos + 2);
            break;
        }

        content.push_str(line.as_str());
        content.push_str("\r\n");
    }

    return Ok(Block { content });
}

fn read_system_block(parser: &mut StringParser) -> Result<Block, ParsingError> {
    let mut is_balanced = false;
    let mut nesting_level = 1;

    let mut cur_content = String::new();

    loop {
        if !parser.has_more() {
            break;
        }

        let c = parser.next().unwrap();
        if c == '{' {
            if nesting_level > 1 {
                return Err(ParsingError {
                    message:
                        "Invalid message format, nested blocks are not supported in system blocks"
                            .to_string(),
                });
            }

            nesting_level += 1;
            cur_content.push(c);
        } else if c == '}' {
            nesting_level -= 1;
            if nesting_level == 0 {
                is_balanced = true;
                break;
            }

            cur_content.push(c);
        } else {
            cur_content.push(c);
        }
    }

    if !is_balanced {
        return Err(ParsingError { message: "Invalid message format, got end of stream while reading system block before closing }".to_string() });
    }

    Ok(Block {
        content: cur_content,
    })
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, NaiveDate, Utc};

    use crate::swift::mt::{
        model::{ApplicationHeader, ServiceIdentifier},
        swift_mt_parser::SwiftMtParser,
    };

    #[test]
    fn positive_tests_parse() {
        let mut msg = "{1:F01FOOBARXXAXXX0000000000}{2:I103FOOBARXXAXXXN}{3:{108:themur}{433:field433}}{5:{PDE:pde}{CHK:chk}}";
        let mut parser = SwiftMtParser::new();
        let mut message = parser.parse(msg.into()).unwrap();

        assert_eq!(
            message.user_header.message_user_reference.unwrap(),
            "themur"
        );
        assert_eq!(
            message.user_header.screening_information_receiver.unwrap(),
            "field433"
        );
        assert_eq!(message.user_header.service_identifier.is_none(), true);
        assert_eq!(message.user_header.banking_priority.is_none(), true);

        assert_eq!(message.trailer.pde.unwrap(), "pde");
        assert_eq!(message.trailer.chk.unwrap(), "chk");
        assert_eq!(message.trailer.pdm.is_none(), true);

        assert_eq!(message.basic_header.application_identifier, "F");
        assert_eq!(
            message.basic_header.service_identifier,
            ServiceIdentifier::Message
        );
        assert_eq!(message.basic_header.logical_terminal, "FOOBARXXAXXX");
        assert_eq!(message.basic_header.session_number, 0);
        assert_eq!(message.basic_header.sequence_number, 0);

        match message.application_header {
            ApplicationHeader::Input { data } => {
                assert_eq!(data.message_type, "103");
                assert_eq!(data.destination, "FOOBARXXAXXX");
                assert_eq!(data.priority, "N");
                assert_eq!(data.delivery_monitoring, "");
                assert_eq!(data.obsolescence_period, "");
            }
            _ => {
                panic!(
                    "Application header is not input but {:?}",
                    message.application_header
                );
            }
        }

        msg = "{1:F01FOOBARXXAXXX0000000000}{2:O0511511010606ABLRXXXXGXXX00000130850106141149S}{3:{108:themur}{433:field433}}{5:{PDE:pde}{CHK:chk}}";
        parser = SwiftMtParser::new();
        message = parser.parse(msg.into()).unwrap();

        match message.application_header {
            ApplicationHeader::Output { data } => {
                assert_eq!(data.message_type, "051");
                assert_eq!(
                    data.sender_datetime,
                    DateTime::<Utc>::from_utc(
                        NaiveDate::from_ymd_opt(2001, 06, 06)
                            .unwrap()
                            .and_hms_opt(15, 11, 0)
                            .unwrap(),
                        Utc
                    )
                );
                assert_eq!(data.sender_address, "ABLRXXXXGXXX");
                assert_eq!(data.session_number, "0000");
                assert_eq!(data.sequence_number, "013085");
                assert_eq!(
                    data.receiver_datetime,
                    DateTime::<Utc>::from_utc(
                        NaiveDate::from_ymd_opt(2001, 6, 14)
                            .unwrap()
                            .and_hms_opt(11, 49, 0)
                            .unwrap(),
                        Utc
                    )
                );
                assert_eq!(data.message_priority, "S");
            }
            _ => {
                panic!(
                    "Application header is not input but {:?}",
                    message.application_header
                );
            }
        }
    }

    #[test]
    fn positive_tests_parse_blocks() {
        let mut msg = "{1:F01FOOBARXXAXXX0000000000}{2:I103FOOBARXXAXXXN}";
        let mut parser = SwiftMtParser::new();
        let mut result = parser.parse_blocks(msg.to_string()).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(
            result.get(&'1').unwrap().content,
            "F01FOOBARXXAXXX0000000000"
        );
        assert_eq!(result.get(&'2').unwrap().content, "I103FOOBARXXAXXXN");

        msg = "{1:F01FOOBARXXAXXX0000000000}{2:I103FOOBARXXAXXXN}{3:{108:asdf}{205:1233}}";
        parser = SwiftMtParser::new();
        result = parser.parse_blocks(msg.to_string()).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(
            result.get(&'1').unwrap().content,
            "F01FOOBARXXAXXX0000000000"
        );
        assert_eq!(result.get(&'2').unwrap().content, "I103FOOBARXXAXXXN");
        assert_eq!(result.get(&'3').unwrap().content, "{108:asdf}{205:1233}");

        msg = "{1:F01FOOBARXXAXXX0000000000}{2:I103FOOBARXXAXXXN}{3:{108:asdf}{205:1233}}{4:\r\n23G:NEWM\r\n-}";
        parser = SwiftMtParser::new();
        result = parser.parse_blocks(msg.to_string()).unwrap();

        assert_eq!(result.len(), 4);
        assert_eq!(
            result.get(&'1').unwrap().content,
            "F01FOOBARXXAXXX0000000000"
        );
        assert_eq!(result.get(&'2').unwrap().content, "I103FOOBARXXAXXXN");
        assert_eq!(result.get(&'3').unwrap().content, "{108:asdf}{205:1233}");
        assert_eq!(result.get(&'4').unwrap().content, "23G:NEWM\r\n");

        msg = "{1:F01FOOBARXXAXXX0000000000}{2:I103FOOBARXXAXXXN}{3:{108:asdf}{205:1233}}{4:\r\n23G:NEWM\r\n-}{5:{CHK:1234567890}}";
        parser = SwiftMtParser::new();
        result = parser.parse_blocks(msg.to_string()).unwrap();

        assert_eq!(result.len(), 5);
        assert_eq!(
            result.get(&'1').unwrap().content,
            "F01FOOBARXXAXXX0000000000"
        );
        assert_eq!(result.get(&'2').unwrap().content, "I103FOOBARXXAXXXN");
        assert_eq!(result.get(&'3').unwrap().content, "{108:asdf}{205:1233}");
        assert_eq!(result.get(&'4').unwrap().content, "23G:NEWM\r\n");
        assert_eq!(result.get(&'5').unwrap().content, "{CHK:1234567890}");

        msg = "{1:F01FOOBARXXAXXX0000000000}{2:I103FOOBARXXAXXXN}{3:{108:asdf}{205:1233}}{4:\r\n-}{5:{CHK:1234567890}}";
        parser = SwiftMtParser::new();
        result = parser.parse_blocks(msg.to_string()).unwrap();

        assert_eq!(result.len(), 5);
        assert_eq!(
            result.get(&'1').unwrap().content,
            "F01FOOBARXXAXXX0000000000"
        );
        assert_eq!(result.get(&'2').unwrap().content, "I103FOOBARXXAXXXN");
        assert_eq!(result.get(&'3').unwrap().content, "{108:asdf}{205:1233}");
        assert_eq!(result.get(&'4').unwrap().content, "");
        assert_eq!(result.get(&'5').unwrap().content, "{CHK:1234567890}");

        msg = "{1:F01FOOBARXXAXXX0000000000}{2:I103FOOBARXXAXXXN}{3:{108:asdf}{205:1233}}{4:\r\n23G:NEWM\r\n20C:SEME//asdf\r\n-}{5:{CHK:1234567890}}";
        parser = SwiftMtParser::new();
        result = parser.parse_blocks(msg.to_string()).unwrap();

        assert_eq!(result.len(), 5);
        assert_eq!(
            result.get(&'1').unwrap().content,
            "F01FOOBARXXAXXX0000000000"
        );
        assert_eq!(result.get(&'2').unwrap().content, "I103FOOBARXXAXXXN");
        assert_eq!(result.get(&'3').unwrap().content, "{108:asdf}{205:1233}");
        assert_eq!(
            result.get(&'4').unwrap().content,
            "23G:NEWM\r\n20C:SEME//asdf\r\n"
        );
        assert_eq!(result.get(&'5').unwrap().content, "{CHK:1234567890}");
    }
}
