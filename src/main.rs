use rs_dbc::Dbc;
use std::fs::File;
use std::io::{self, Read};

fn main() -> io::Result<()> {
    let mut f = File::open("./examples/sample.dbc")?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)?;

    let dbc = Dbc::from_slice(&buffer).expect("Failed to parse DBC file");

    for msg in dbc.messages {
        println!("Message Name: {}", msg.message_name);
        println!("Message ID: 0x{:X}", msg.message_id.raw());
        println!("Size: {}", msg.message_size);
        println!("Cycle Time: {}", msg.cycle_time);
        println!("Transmitter: {}", msg.transmitter);
        println!("ID-Format: {}", msg.message_id.kind());

        for sig in msg.signals {
            println!("Signal Name: {}", sig.name);
            //println!("Start Bit: {}", sig.start_bit);
            //println!("Vector Start Bit: {}", sig.vector_start_bit());
            // println!("Signal Size: {}", sig.signal_size);
            // println!("Byte Order: {}", sig.byte_order);
            println!("Value Type: {}", sig.value_type);
            // println!("Factor: {}", sig.factor);
            // println!("Offset: {}", sig.offset);
            // println!("Min: {}", sig.min);
            // println!("Max: {}", sig.max);
            // println!("Unit: {}", sig.unit);
            // println!("Receivers: {}", sig.receivers.join(", "));
            // println!("Initial Value: {}", sig.initial_value);
            // println!("Vector Initial Value: {}", sig.vector_initial_value());
            // println!("Vector Value Descriptions: {}", sig.vector_value_descriptions().iter().map(|(k, v)| format!("{}: {}", k, v)).collect::<Vec<String>>().join(", "));
            //println!("Multiplexer Type: {}", sig.multiplexer_type);
            // println!("Send Type: {}", msg.tx_method);
            // println!("Value Descriptions: {}", sig.value_descriptions().iter().map(|(k, v)| format!("{}: {}", k, v)).collect::<Vec<String>>().join(", "));
            println!("");
        }
        println!("");
    }
    Ok(())
}
