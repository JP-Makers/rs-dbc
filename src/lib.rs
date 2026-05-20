//!
//! rs-dbc is a library written in Rust for parsing and handling CAN DBC files.
//!
//! ```rust
//! use std::fs::File;
//! use std::io::{self, Read};
//! use rs_dbc::Dbc;
//!
//! fn main() -> io::Result<()> {
//!     let mut f = File::open("./examples/simple.dbc")?;
//!     let mut buffer = Vec::new();
//!     f.read_to_end(&mut buffer)?;
//!
//!     let dbc = Dbc::from_slice(&buffer).expect("Failed to parse DBC file");
//!
//!     for msg in dbc.messages {
//!         println!("Message Name: {}", msg.message_name);
//!         println!("Message ID: 0x{:X}", msg.message_id.raw());
//!         println!("ID-Format: {}", msg.message_id.kind());
//!
//!         for sig in msg.signals {
//!             println!("Signal Name: {}", sig.name);
//!             println!("Byte Order: {}", sig.byte_order);
//!             println!("Value Type: {}", sig.value_type);
//!             println!("");
//!         }
//!         println!("");
//!     }
//!     Ok(())
//! }
//! ```

use regex::Regex;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::str;

use std::fmt;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
    Invalid(Dbc, String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Invalid(_, msg) => write!(f, "Invalid DBC input: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MessageID {
    Standard(u16),
    Extended(u32),
}

impl MessageID {
    pub fn raw(&self) -> u32 {
        match self {
            MessageID::Standard(id) => *id as u32,
            MessageID::Extended(id) => *id | (1 << 31),
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            MessageID::Standard(_) => "CAN Standard",
            MessageID::Extended(_) => "CAN Extended",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ByteOrder {
    Intel,
    Motorola,
}

impl fmt::Display for ByteOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ByteOrder::Intel => write!(f, "Intel"),
            ByteOrder::Motorola => write!(f, "Motorola"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueType {
    Signed,
    Unsigned,
    Float,
    Double,
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueType::Signed => write!(f, "Signed"),
            ValueType::Unsigned => write!(f, "Unsigned"),
            ValueType::Float => write!(f, "IEEE Float"),
            ValueType::Double => write!(f, "IEEE Double"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MultiplexIndicator {
    Plain,
    Multiplexer,
    Multiplexed,
}

impl fmt::Display for MultiplexIndicator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MultiplexIndicator::Plain => write!(f, "Plain"),
            MultiplexIndicator::Multiplexer => write!(f, "Multiplexer"),
            MultiplexIndicator::Multiplexed => write!(f, "Multiplexed"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Signal {
    pub name: String,
    pub start_bit: u64,
    pub signal_size: u64,
    pub byte_order: ByteOrder,
    pub value_type: ValueType,
    pub factor: f64,
    pub offset: f64,
    pub min: f64,
    pub max: f64,
    pub unit: String,
    pub receivers: Vec<String>,
    pub value_descriptions: HashMap<u64, String>,
    pub multiplexer_type: MultiplexIndicator,
    pub initial_value: f64,
}

impl Signal {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn start_bit(&self) -> u64 {
        self.start_bit
    }

    /// Returns the start bit as displayed in Vector CANdb++
    pub fn vector_start_bit(&self) -> u64 {
        match self.byte_order {
            ByteOrder::Intel => self.start_bit,
            ByteOrder::Motorola => {
                let start_byte = self.start_bit / 8;
                let start_bit_in_byte = self.start_bit % 8;
                let end_bit_1 = self.start_bit + 1;
                let end_bit_2 = self.start_bit.saturating_sub(self.signal_size - 1);
                let end_byte = end_bit_1 / 8;

                if start_byte != end_byte && self.signal_size > 8 {
                    end_bit_1
                } else if start_bit_in_byte != 8 {
                    end_bit_2
                } else {
                    self.start_bit
                }
            }
        }
    }

    pub fn signal_size(&self) -> u64 {
        self.signal_size
    }

    pub fn byte_order(&self) -> ByteOrder {
        self.byte_order
    }

    pub fn value_type(&self) -> ValueType {
        self.value_type
    }

    pub fn factor(&self) -> f64 {
        self.factor
    }

    pub fn offset(&self) -> f64 {
        self.offset
    }

    pub fn min(&self) -> f64 {
        self.min
    }

    pub fn max(&self) -> f64 {
        self.max
    }

    pub fn unit(&self) -> &str {
        &self.unit
    }

    pub fn receivers(&self) -> &Vec<String> {
        &self.receivers
    }

    pub fn value_descriptions(&self) -> &HashMap<u64, String> {
        &self.value_descriptions
    }

    pub fn multiplexer_type(&self) -> MultiplexIndicator {
        self.multiplexer_type
    }

    pub fn initial_value(&self) -> f64 {
        self.initial_value
    }

    /// Returns the initial value as displayed in Vector CANdb++
    /// Formula: (Raw value × factor) + offset
    pub fn vector_initial_value(&self) -> f64 {
        (self.initial_value * self.factor) + self.offset
    }

    /// Returns the value descriptions sorted by value and formatted as hex strings
    pub fn vector_value_descriptions(&self) -> Vec<(String, String)> {
        let mut desc: Vec<_> = self.value_descriptions.iter().collect();
        desc.sort_by_key(|a| a.0);
        desc.into_iter()
            .map(|(&k, v)| (format!("0x{:X}", k), v.clone()))
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Message {
    pub message_name: String,
    pub message_id: MessageID,
    pub message_size: u64,
    pub cycle_time: u32,
    pub transmitter: String,
    pub tx_method: String,
    pub signals: Vec<Signal>,
}

impl Message {
    pub fn message_name(&self) -> &str {
        &self.message_name
    }

    pub fn message_id(&self) -> (u32, &'static str) {
        (self.message_id.raw(), self.message_id.kind())
    }

    pub fn message_size(&self) -> u64 {
        self.message_size
    }

    pub fn cycle_time(&self) -> u32 {
        self.cycle_time
    }

    pub fn transmitter(&self) -> &str {
        if self.transmitter.starts_with("Vector__XXX") {
            "No Transmitter"
        } else {
            &self.transmitter
        }
    }

    pub fn tx_method(&self) -> &str {
        &self.tx_method
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dbc {
    pub messages: Vec<Message>,
}

impl Dbc {
    pub fn from_slice(buffer: &[u8]) -> Result<Dbc, Error> {
        let dbc_input = str::from_utf8(buffer).unwrap();
        Self::try_from(dbc_input)
    }

    pub fn from_slice_lossy(buffer: &[u8]) -> Result<Dbc, Error> {
        let dbc_input = String::from_utf8_lossy(buffer);
        Self::try_from(dbc_input.as_ref())
    }
}

impl TryFrom<&str> for Dbc {
    type Error = Error;

    fn try_from(dbc_input: &str) -> Result<Self, Self::Error> {
        let messages = parse_message(dbc_input);

        if messages.is_empty() {
            return Err(Error::Invalid(Dbc { messages }, dbc_input.to_string()));
        }
        Ok(Dbc { messages })
    }
}

fn parse_message(dbc_input: &str) -> Vec<Message> {
    let message_names = parse_message_name(dbc_input);
    let message_size = parse_message_size(dbc_input);
    let message_transmitters = parse_message_transmitters(dbc_input);
    let default_cycles = parse_default_cycle_time(dbc_input).unwrap_or(0);
    let explicit_cycles = parse_explicit_cycle_time(dbc_input);
    let (default_tx_method, explicit_tx_methods) = parse_tx_methods(dbc_input);
    let value_descriptions = parse_value_descriptions(dbc_input);
    let signals = parse_signals(dbc_input, &value_descriptions);

    let mut message = Vec::new();

    for (id, message_name) in message_names {
        let cycle_time = explicit_cycles.get(&id).copied().unwrap_or(default_cycles);
        let message_size = message_size.get(&id).copied().unwrap_or(0);
        let message_signals = signals.get(&id).cloned().unwrap_or_else(Vec::new);
        let transmitter = message_transmitters
            .get(&id)
            .cloned()
            .unwrap_or_else(|| "Vector__XXX".to_string());
        let tx_method = explicit_tx_methods
            .get(&id)
            .cloned()
            .unwrap_or_else(|| default_tx_method.clone());

        let message_id = if id < 0x800 {
            MessageID::Standard(id as u16)
        } else {
            MessageID::Extended(id)
        };

        message.push(Message {
            message_name,
            message_id,
            message_size,
            cycle_time,
            transmitter,
            tx_method,
            signals: message_signals,
        });
    }

    message
}

fn parse_message_name(dbc_input: &str) -> HashMap<u32, String> {
    let re_name = Regex::new(r#"BO_\s+(\d+)\s+(\w+):"#).unwrap();
    let mut map = HashMap::new();

    for cap in re_name.captures_iter(dbc_input) {
        if let (Ok(id), Ok(name)) = (cap[1].parse::<u32>(), cap[2].parse::<String>()) {
            map.insert(id, name);
        }
    }
    map
}

fn parse_message_size(dbc_input: &str) -> HashMap<u32, u64> {
    let re_size = Regex::new(r#"BO_\s+(\d+)\s+\w+:\s+(\d+)"#).unwrap();
    let mut map = HashMap::new();

    for cap in re_size.captures_iter(dbc_input) {
        if let (Ok(id), Ok(size)) = (cap[1].parse::<u32>(), cap[2].parse::<u64>()) {
            map.insert(id, size);
        }
    }
    map
}

fn parse_message_transmitters(dbc_input: &str) -> HashMap<u32, String> {
    let re_transmitter = Regex::new(r#"BO_\s+(\d+)\s+\w+:\s+\d+\s+(\w+)"#).unwrap();
    let mut map = HashMap::new();

    for cap in re_transmitter.captures_iter(dbc_input) {
        if let Ok(id) = cap[1].parse::<u32>() {
            let transmitter = cap[2].to_string();
            map.insert(id, transmitter);
        }
    }
    map
}

fn parse_default_cycle_time(dbc_input: &str) -> Option<u32> {
    let re_default = Regex::new(r#"BA_DEF_DEF_\s+"GenMsgCycleTime"\s+(\d+);"#).unwrap();
    if let Some(cap) = re_default.captures(dbc_input) {
        return cap[1].parse::<u32>().ok();
    }
    None
}

fn parse_explicit_cycle_time(dbc_input: &str) -> HashMap<u32, u32> {
    let re_explicit = Regex::new(r#"BA_ "GenMsgCycleTime" BO_ (\d+) (\d+);"#).unwrap();
    let mut map = HashMap::new();

    for cap in re_explicit.captures_iter(dbc_input) {
        if let (Ok(id), Ok(cycle)) = (cap[1].parse::<u32>(), cap[2].parse::<u32>()) {
            map.insert(id, cycle);
        }
    }
    map
}

fn parse_signals(
    dbc_input: &str,
    value_descriptions: &HashMap<(u32, String), HashMap<u64, String>>,
) -> HashMap<u32, Vec<Signal>> {
    let re_signal = Regex::new(r#"SG_\s+(\w+)\s*([mM]?\d*)\s*:\s*(\d+)\|(\d+)@([01])([+-])\s*\(([^,]+),([^)]+)\)\s*\[([^|]+)\|([^\]]+)\]\s*"([^"]*)"\s*(.*)"#).unwrap();
    let initial_values = parse_initial_values(dbc_input);
    let sig_valtypes = parse_sig_valtypes(dbc_input);
    let mut signals_map: HashMap<u32, Vec<Signal>> = HashMap::new();
    let mut current_message_id = 0u32;
    let lines: Vec<&str> = dbc_input.lines().collect();

    for line in lines {
        if let Some(msg_cap) = Regex::new(r#"BO_\s+(\d+)\s+\w+:"#).unwrap().captures(line)
            && let Ok(id) = msg_cap[1].parse::<u32>()
        {
            current_message_id = id;
            signals_map.entry(current_message_id).or_default();
        }

        if let Some(cap) = re_signal.captures(line)
            && let (Ok(start_bit), Ok(signal_size), Ok(factor), Ok(offset), Ok(min), Ok(max)) = (
                cap[3].parse::<u64>(),
                cap[4].parse::<u64>(),
                cap[7].parse::<f64>(),
                cap[8].parse::<f64>(),
                cap[9].parse::<f64>(),
                cap[10].parse::<f64>(),
            )
        {
            let signal_name = cap[1].to_string();
            let byte_order = if &cap[5] == "1" {
                ByteOrder::Intel
            } else {
                ByteOrder::Motorola
            };
            let mut value_type = if &cap[6] == "+" {
                ValueType::Unsigned
            } else {
                ValueType::Signed
            };

            if let Some(valtype) = sig_valtypes.get(&(current_message_id, signal_name.clone())) {
                if *valtype == 1 {
                    value_type = ValueType::Float;
                } else if *valtype == 2 {
                    value_type = ValueType::Double;
                }
            }

            // Parse multiplexer information
            let multiplexer_info = cap[2].to_string();
            let multiplexer_type = if multiplexer_info.is_empty() {
                MultiplexIndicator::Plain
            } else if multiplexer_info == "M" {
                MultiplexIndicator::Multiplexer
            } else if multiplexer_info.starts_with("m") {
                MultiplexIndicator::Multiplexed
            } else {
                MultiplexIndicator::Plain
            };

            // Parse receivers from the end of the line
            let receivers_str = cap.get(12).map_or("", |m| m.as_str()).trim();
            let receivers: Vec<String> = if receivers_str.is_empty() {
                Vec::new()
            } else {
                receivers_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect()
            };

            let signal_value_descriptions = value_descriptions
                .get(&(current_message_id, signal_name.clone()))
                .cloned()
                .unwrap_or_default();

            let initial_value = initial_values
                .get(&(current_message_id, signal_name.clone()))
                .copied()
                .unwrap_or(0.0);

            let signal = Signal {
                name: signal_name,
                start_bit,
                signal_size,
                byte_order,
                value_type,
                factor,
                offset,
                min,
                max,
                unit: cap[11].to_string(),
                receivers,
                value_descriptions: signal_value_descriptions,
                multiplexer_type,
                initial_value,
            };

            if let Some(signals) = signals_map.get_mut(&current_message_id) {
                signals.push(signal);
            }
        }
    }

    signals_map
}

fn parse_initial_values(dbc_input: &str) -> HashMap<(u32, String), f64> {
    let re_sig_val =
        Regex::new(r#"BA_\s+"GenSigStartValue"\s+SG_\s+(\d+)\s+([^\s]+)\s+([^;]+);"#).unwrap();
    let mut initial_values: HashMap<(u32, String), f64> = HashMap::new();

    for cap in re_sig_val.captures_iter(dbc_input) {
        if let Ok(message_id) = cap[1].parse::<u32>() {
            let signal_name = cap[2].to_string();
            if let Ok(value) = cap[3].trim().parse::<f64>() {
                initial_values.insert((message_id, signal_name), value);
            }
        }
    }

    initial_values
}

fn parse_sig_valtypes(dbc_input: &str) -> HashMap<(u32, String), u32> {
    let re_valtype =
        Regex::new(r#"SIG_VALTYPE_\s+(\d+)\s+([a-zA-Z0-9_]+)\s*:\s*(\d+)\s*;"#).unwrap();
    let mut map = HashMap::new();
    for cap in re_valtype.captures_iter(dbc_input) {
        if let (Ok(msg_id), Ok(val_type)) = (cap[1].parse::<u32>(), cap[3].parse::<u32>()) {
            map.insert((msg_id, cap[2].to_string()), val_type);
        }
    }
    map
}

fn parse_value_descriptions(dbc_input: &str) -> HashMap<(u32, String), HashMap<u64, String>> {
    let re_val = Regex::new(r#"VAL_\s+(\d+)\s+(\w+)\s+(.+?);"#).unwrap();
    let mut value_descriptions: HashMap<(u32, String), HashMap<u64, String>> = HashMap::new();

    for cap in re_val.captures_iter(dbc_input) {
        if let Ok(message_id) = cap[1].parse::<u32>() {
            let signal_name = cap[2].to_string();
            let values_str = &cap[3];

            let mut signal_values = HashMap::new();
            let re_value_pair = Regex::new(r#"(\d+)\s+"([^"]+)""#).unwrap();

            for value_cap in re_value_pair.captures_iter(values_str) {
                if let Ok(value) = value_cap[1].parse::<u64>() {
                    let description = value_cap[2].to_string();
                    signal_values.insert(value, description);
                }
            }

            if !signal_values.is_empty() {
                value_descriptions.insert((message_id, signal_name), signal_values);
            }
        }
    }

    value_descriptions
}

fn parse_tx_methods(dbc_input: &str) -> (String, HashMap<u32, String>) {
    let re_enum = Regex::new(r#"BA_DEF_\s+BO_\s+"GenMsgSendType"\s+ENUM\s+([^;]+);"#).unwrap();
    let mut enum_variants = Vec::new();
    if let Some(cap) = re_enum.captures(dbc_input) {
        let enums_str = &cap[1];
        enum_variants = enums_str
            .split(',')
            .map(|s| s.trim().trim_matches('"').to_string())
            .collect();
    }

    let mut default_method = String::new();
    let re_default_str = Regex::new(r#"BA_DEF_DEF_\s+"GenMsgSendType"\s+"([^"]+)";"#).unwrap();
    if let Some(cap) = re_default_str.captures(dbc_input) {
        default_method = cap[1].to_string();
    } else {
        let re_default_int = Regex::new(r#"BA_DEF_DEF_\s+"GenMsgSendType"\s+(\d+);"#).unwrap();
        if let Some(cap) = re_default_int.captures(dbc_input) {
            if let Ok(idx) = cap[1].parse::<usize>() {
                if idx < enum_variants.len() {
                    default_method = enum_variants[idx].clone();
                }
            }
        }
    }

    let re_explicit = Regex::new(r#"BA_\s+"GenMsgSendType"\s+BO_\s+(\d+)\s+([^;]+);"#).unwrap();
    let mut map = HashMap::new();

    for cap in re_explicit.captures_iter(dbc_input) {
        if let Ok(id) = cap[1].parse::<u32>() {
            let val_str = cap[2].trim();
            if val_str.starts_with('"') {
                map.insert(id, val_str.trim_matches('"').to_string());
            } else if let Ok(idx) = val_str.parse::<usize>() {
                if idx < enum_variants.len() {
                    map.insert(id, enum_variants[idx].clone());
                } else {
                    map.insert(id, val_str.to_string());
                }
            }
        }
    }

    (default_method, map)
}
