use std::io::Write;
use tabwriter::TabWriter;

pub fn write_tab_written_message(message: String) {
    let mut tw = TabWriter::new(vec![]);
    tw.write_all(message.as_bytes()).unwrap();
    tw.flush().unwrap();
    println!("{}", String::from_utf8(tw.into_inner().unwrap()).unwrap());
}
