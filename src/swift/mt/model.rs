use crate::utils::StringParser;
use chrono::DateTime;
use chrono::NaiveDateTime;
use chrono::Utc;
use std::collections::HashMap;
use std::io;
use std::io::ErrorKind::InvalidData;

#[derive(FromPrimitive, Debug, PartialEq)]
pub enum ServiceIdentifier {
    Message = 1,
    LoginRequest = 2,
    Select = 3,
    Quit = 5,
    Logout = 6,
    RemoveTerminalRequest = 14,
    SystemLogout = 16,
    MessageAck = 21,
    LoginAck = 22,
    SelectAck = 23,
    QuitAck = 25,
    LogoutAck = 26,
    LoginNegativeAck = 42,
    SelectNegativeAck = 43,
}

pub struct BasicHeader {
    pub application_identifier: String,
    pub service_identifier: ServiceIdentifier,
    pub logical_terminal: String,
    pub session_number: u32,
    pub sequence_number: u32,
}

impl BasicHeader {
    pub fn new() -> BasicHeader {
        return BasicHeader {
            application_identifier: "F".into(),
            service_identifier: ServiceIdentifier::Message,
            logical_terminal: "            ".into(),
            session_number: 0,
            sequence_number: 0,
        };
    }

    pub fn from_raw(parser: &mut StringParser) -> Result<BasicHeader, io::Error> {
        let application_identifier = parser.n_chars(1)?;
        let service_identifier_raw = parser.n_chars(2)?;
        let logical_terminal = parser.n_chars(12)?;
        let session_number: u32 = parser
            .n_chars(4)?
            .parse::<u32>()
            .map_err(|e| io::Error::new(InvalidData, e))?;
        let sequence_number: u32 = parser
            .n_chars(6)?
            .parse::<u32>()
            .map_err(|e| io::Error::new(InvalidData, e))?;

        let service_identifier_num = service_identifier_raw.parse::<u32>().map_err(|e| {
            io::Error::new(
                InvalidData,
                format!(
                    "Could not convert {} to a service identifier: {:?}",
                    service_identifier_raw, e
                ),
            )
        })?;
        let service_identifier: ServiceIdentifier =
            num::FromPrimitive::from_u32(service_identifier_num).ok_or(io::Error::new(
                InvalidData,
                format!(
                    "Unknown value for enum ServiceIdentifier: {}",
                    service_identifier_num
                ),
            ))?;

        return Ok(BasicHeader {
            application_identifier,
            service_identifier,
            logical_terminal,
            session_number,
            sequence_number,
        });
    }
}

#[derive(Debug)]
pub enum ApplicationHeader {
    Input { data: InputData },
    Output { data: OutputData },
    Empty,
}

#[derive(Debug)]
pub struct InputData {
    pub message_type: String,
    pub destination: String,
    pub priority: String,
    pub delivery_monitoring: String,
    pub obsolescence_period: String,
}

#[derive(Debug)]
pub struct OutputData {
    pub message_type: String,
    pub sender_datetime: DateTime<Utc>,
    pub sender_address: String,
    pub session_number: String,
    pub sequence_number: String,
    pub receiver_datetime: DateTime<Utc>,
    pub message_priority: String,
}

impl ApplicationHeader {
    pub fn from_raw(parser: &mut StringParser) -> Result<ApplicationHeader, io::Error> {
        let direction = parser.next()?;
        let message_type = parser.n_chars(3)?;

        return if direction == 'I' {
            let destination = parser.n_chars(12)?;
            let priority = parser.n_chars(1)?;
            let delivery_monitoring = parser.n_chars(1).ok().unwrap_or("".to_string());
            let obsolescence_period = parser.n_chars(3).ok().unwrap_or("".to_string());
            Ok(ApplicationHeader::Input {
                data: InputData {
                    message_type,
                    destination,
                    priority,
                    delivery_monitoring,
                    obsolescence_period,
                },
            })
        } else if direction == 'O' {
            let sender_time = parser.n_chars(4)?;
            let sender_date = parser.n_chars(6)?;

            let sender_date_time = NaiveDateTime::parse_from_str(
                format!("{}{}", sender_date, sender_time).as_str(),
                "%y%m%d%H%M",
            )
            .map_err(|e| {
                io::Error::new(
                    InvalidData,
                    format!("Cannot parse sender date/time: {}", e.to_string()),
                )
            })?;
            let sender_address = parser.n_chars(12)?;
            let session_number = parser.n_chars(4)?;
            let sequence_number = parser.n_chars(6)?;

            let receiver_date = parser.n_chars(6)?;
            let receiver_time = parser.n_chars(4)?;
            let receiver_date_time = NaiveDateTime::parse_from_str(
                format!("{}{}", receiver_date, receiver_time).as_str(),
                "%y%m%d%H%M",
            )
            .map_err(|e| {
                io::Error::new(
                    InvalidData,
                    format!("Cannot parse receiver date/time: {}", e.to_string()),
                )
            })?;

            let message_priority = parser.n_chars(1)?;
            Ok(ApplicationHeader::Output {
                data: OutputData {
                    message_type,
                    sender_datetime: DateTime::<Utc>::from_utc(sender_date_time, Utc),
                    sender_address,
                    session_number,
                    sequence_number,
                    receiver_datetime: DateTime::<Utc>::from_utc(receiver_date_time, Utc),
                    message_priority,
                },
            })
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid direction: {}", direction),
            ))
        };
    }
}

pub struct UserHeader {
    /* 103 */ pub service_identifier: Option<String>,
    /* 113 */ pub banking_priority: Option<String>,
    /* 108 */ pub message_user_reference: Option<String>,
    /* 119 */ pub validation_flag: Option<String>,
    /* 423 */ pub balance_checkpoint_date_time: Option<String>,
    /* 106 */ pub mir: Option<String>,
    /* 424 */ pub related_reference: Option<String>,
    /* 111 */ pub service_type_identifier: Option<String>,
    /* 121 */ pub uetr: Option<String>,
    /* 115 */ pub payment_release_information_receiver_fin_copy: Option<String>,
    /* 165 */ pub payment_release_information_receiver_fin_inform: Option<String>,
    /* 433 */ pub screening_information_receiver: Option<String>,
    /* 434 */ pub payment_controls_information_for_receiver: Option<String>,
    pub unk_fields: HashMap<String, String>,
}

impl UserHeader {
    pub fn new() -> UserHeader {
        return UserHeader {
            service_identifier: Option::None,
            banking_priority: Option::None,
            message_user_reference: Option::None,
            validation_flag: Option::None,
            balance_checkpoint_date_time: Option::None,
            mir: Option::None,
            related_reference: Option::None,
            service_type_identifier: Option::None,
            uetr: Option::None,
            payment_release_information_receiver_fin_copy: Option::None,
            payment_release_information_receiver_fin_inform: Option::None,
            screening_information_receiver: Option::None,
            payment_controls_information_for_receiver: Option::None,
            unk_fields: HashMap::new(),
        };
    }

    pub fn from_raw(content: String) -> Result<UserHeader, io::Error> {
        let mut fields = read_sys_block_fields(content);
        return Ok(UserHeader {
            service_identifier: fields.remove("103"),
            banking_priority: fields.remove("113"),
            message_user_reference: fields.remove("108"),
            validation_flag: fields.remove("119"),
            balance_checkpoint_date_time: fields.remove("423"),
            mir: fields.remove("106"),
            related_reference: fields.remove("424"),
            service_type_identifier: fields.remove("111"),
            uetr: fields.remove("121"),
            payment_release_information_receiver_fin_copy: fields.remove("115"),
            payment_release_information_receiver_fin_inform: fields.remove("165"),
            screening_information_receiver: fields.remove("433"),
            payment_controls_information_for_receiver: fields.remove("434"),
            unk_fields: fields,
        });
    }

    pub fn to_raw(self) -> String {
        let mut ret = String::new();
        ret.push_str("{3:");

        if let Some(service_identifier) = self.service_identifier {
            ret.push_str(&format!("{{103:{service_identifier}}}"))
        }

        if let Some(banking_priority) = self.banking_priority {
            ret.push_str(&format!("{{113:{banking_priority}}}"))
        }

        if let Some(message_user_reference) = self.message_user_reference {
            ret.push_str(&format!("{{108:{message_user_reference}}}"))
        }

        if let Some(validation_flag) = self.validation_flag {
            ret.push_str(&format!("{{119:{validation_flag}}}"))
        }

        if let Some(balance_checkpoint_date_time) = self.balance_checkpoint_date_time {
            ret.push_str(&format!("{{423:{balance_checkpoint_date_time}}}"))
        }

        if let Some(mir) = self.mir {
            ret.push_str(&format!("{{106:{mir}}}"))
        }

        if let Some(related_reference) = self.related_reference {
            ret.push_str(&format!("{{424:{related_reference}}}"))
        }

        if let Some(service_type_identifier) = self.service_type_identifier {
            ret.push_str(&format!("{{111:{service_type_identifier}}}"))
        }

        if let Some(uetr) = self.uetr {
            ret.push_str(&format!("{{121:{uetr}}}"))
        }

        if let Some(payment_release_information_receiver_fin_copy) =
            self.payment_release_information_receiver_fin_copy
        {
            ret.push_str(&format!(
                "{{115:{payment_release_information_receiver_fin_copy}}}"
            ))
        }

        if let Some(payment_release_information_receiver_fin_inform) =
            self.payment_release_information_receiver_fin_inform
        {
            ret.push_str(&format!(
                "{{165:{payment_release_information_receiver_fin_inform}}}"
            ))
        }

        if let Some(screening_information_receiver) = self.screening_information_receiver {
            ret.push_str(&format!("{{433:{screening_information_receiver}}}"))
        }

        if let Some(payment_controls_information_for_receiver) =
            self.payment_controls_information_for_receiver
        {
            ret.push_str(&format!(
                "{{434:{payment_controls_information_for_receiver}}}"
            ))
        }

        self.unk_fields
            .iter()
            .for_each(|(k, v)| ret.push_str(&format!("{{{k}:{v}}}")));

        ret.push_str("}");
        return ret;
    }
}

pub struct Trailer {
    pub pac: Option<String>,
    pub chk: Option<String>,
    pub sys: Option<String>,
    pub tng: Option<String>,
    pub pde: Option<String>,
    pub pdm: Option<String>,
    pub dlm: Option<String>,
    pub mrf: Option<String>,
    pub unk_fields: HashMap<String, String>,
}

impl Trailer {
    pub fn new() -> Trailer {
        return Trailer {
            pac: Option::None,
            chk: Option::None,
            sys: Option::None,
            tng: Option::None,
            pde: Option::None,
            pdm: Option::None,
            dlm: Option::None,
            mrf: Option::None,
            unk_fields: HashMap::new(),
        };
    }

    pub fn from_raw(msg: String) -> Result<Trailer, io::Error> {
        let mut fields = read_sys_block_fields(msg);

        return Ok(Trailer {
            pac: fields.remove("PAC"),
            chk: fields.remove("CHK"),
            sys: fields.remove("SYS"),
            tng: fields.remove("TNG"),
            pde: fields.remove("PDE"),
            pdm: fields.remove("PDM"),
            dlm: fields.remove("DLM"),
            mrf: fields.remove("MRF"),
            unk_fields: fields,
        });
    }

    pub fn to_raw(self) -> String {
        let mut ret = String::new();
        ret.push_str("{3:");

        if let Some(pac) = self.pac {
            ret.push_str(&format!("{{PAC:{pac}}}"));
        }

        if let Some(chk) = self.chk {
            ret.push_str(&format!("{{CHK:{chk}}}"));
        }

        if let Some(sys) = self.sys {
            ret.push_str(&format!("{{SYS:{sys}}}"));
        }

        if let Some(tng) = self.tng {
            ret.push_str(&format!("{{TNG:{tng}}}"));
        }

        if let Some(pde) = self.pde {
            ret.push_str(&format!("{{PDE:{pde}}}"));
        }

        if let Some(pdm) = self.pdm {
            ret.push_str(&format!("{{PDM:{pdm}}}"));
        }

        if let Some(dlm) = self.dlm {
            ret.push_str(&format!("{{DLM:{dlm}}}"));
        }

        if let Some(mrf) = self.mrf {
            ret.push_str(&format!("{{MRF:{mrf}}}"));
        }

        self.unk_fields
            .iter()
            .for_each(|(k, v)| ret.push_str(&format!("{{{k}:{v}}}")));

        ret.push_str("}");

        return ret;
    }
}

fn read_sys_block_fields(content: String) -> HashMap<String, String> {
    return content
        .split("}")
        .into_iter()
        .filter(|tk| !tk.trim().is_empty())
        .map(|tk| &tk[1..])
        .map(|tk| tk.split_once(":").unwrap_or((tk, "")))
        .map(|tk| (tk.0.to_string(), tk.1.to_string()))
        .collect();
}
