use std::{
    fmt::Debug,
    io::{self, BufRead, BufReader},
    process::{Command, Stdio},
};

use thiserror::Error;

use self::types::{Record, Records};

pub mod types;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Input/Output error: {0:#?}")]
    InputOutput(#[from] io::Error),
    #[error("JSON decode error: {0:#?}")]
    JsonDecode(#[from] serde_json::Error),
    #[error("Invalid Records Vec")]
    InvalidRecordVec,
}

pub struct Speedtest {
    server_id: u32,
}

impl Speedtest {
    pub fn new(server_id: u32) -> Self {
        Self { server_id }
    }

    pub fn measure(self) -> Result<Records, Error> {
        let server_id = self.server_id.to_string();
        let args = vec!["-s", &server_id, "--format=jsonl"];

        let stdout = Command::new("speedtest")
            .args(args)
            .stdout(Stdio::piped())
            .spawn()?
            .stdout
            .take()
            .unwrap();

        let reader = BufReader::new(stdout);

        let records = reader.lines().filter_map(|line| line.ok()).map(|line| {
            let record = Record::try_from(line.as_str());

            // debug
            match &record {
                Ok(_) => (),
                Err(error) => println!("[speedtest] got error from speedtest: {:?}", error),
            }

            record
        });

        let records: Result<Vec<Record>, Error> = records.collect();
        let records_vec: Vec<Record> = records?;

        let records = Records::try_from(records_vec)?;

        println!("[speedtest] got result: {:?}", records.result);

        Ok(records)
    }
}
