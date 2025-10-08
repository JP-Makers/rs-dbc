use rs_dbc::{self};
use clap::{command, Arg};
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
                .default_value("./examples/simple.dbc")
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
            for sig in msg.signals {
                println!("Signal Name: {}", sig.name);
                println!("Vector Bit: {}", sig.vector_start_bit());
            }
        }
    Ok(())
}