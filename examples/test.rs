use clap::{Arg, command};
use rs_dbc::{self};
use std::fs::File;
use std::io::{self, Read};

fn main() -> io::Result<()> {
    let matches = command!()
        .version("1.0")
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .value_name("FILE")
                .help("DBC file path")
                .default_value("./examples/sample.dbc")
                .num_args(1),
        )
        .get_matches();
    let path = matches.get_one::<String>("input").unwrap();

    let mut f = File::open(path)?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)?;
    let dbc_in = std::str::from_utf8(&buffer).unwrap();

    let dbc = rs_dbc::Dbc::try_from(dbc_in);
    for msg in dbc.unwrap().messages {
        println!("Message Name: {}", msg.message_name);
        println!("Message ID: 0x{:X}", msg.message_id.raw());
        println!("Size: {}", msg.message_size);
        println!("Cycle Time: {}", msg.cycle_time);
        println!("Transmitter: {}", msg.transmitter);
        println!("ID-Format: {}", msg.message_id.kind());
        println!("Send Type: {}", msg.tx_method);
        println!("");
        for sig in msg.signals {
            println!("Signal Name: {}", sig.name);
            println!("Receivers: {}", sig.receivers.join(", "));
            println!("Start Bit: {}", sig.start_bit);
            println!("Vector Bit: {}", sig.vector_start_bit());
            println!(
                "Vector Value Descriptions: {}",
                sig.vector_value_descriptions()
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect::<Vec<String>>()
                    .join(", ")
            );
            println!("Vector Initial Value: {}", sig.vector_initial_value());
            println!("");
        }
    }
    Ok(())
}
