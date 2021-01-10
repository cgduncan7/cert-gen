use std::io;
use std::fs::File;
use std::collections::HashMap;

pub type Record = HashMap<String, String>;

pub fn read_data(data_path: &str) -> io::Result<Vec<Record>>{
    let file = File::open(data_path)?;
    let mut reader = csv::ReaderBuilder::new()
        .from_reader(file);
    let mut records: Vec<Record> = Vec::new();
    for result in reader.deserialize() {
        let record: Record = result?;
        records.push(record);
    }
    Ok(records)
}