use std::path::Path;

use crossbeam::channel::{unbounded, Receiver, Sender};
use rusqlite::{params, Connection, Error};

use crate::speedtest::types::{Records, StartRecord};

pub struct DatabaseWorker {
    connection: Connection,
    rx: Receiver<Records>,
}

impl DatabaseWorker {
    pub fn new() -> Result<(Self, DatabaseHandle), Error> {
        let path = Path::new("./results.db");
        let needs_init = !path.exists();

        let connection = Connection::open("./results.db")?;

        if needs_init {
            println!("[database] importing schema...");
            connection.execute_batch(include_str!("./schema.sql"))?;
        }

        let (tx, rx) = unbounded();

        let worker = Self { connection, rx };
        let handle = DatabaseHandle { tx };

        Ok((worker, handle))
    }

    pub fn run(self) {
        loop {
            while let Ok(records) = self.rx.recv() {
                self.insert_records(records).unwrap();
            }
        }
    }

    fn insert_records(&self, records: Records) -> Result<(), Error> {
        let measurement_id = self.insert_measurement(&records.start)?;

        self.insert_record(
            measurement_id,
            "start",
            serde_json::to_string(&records.start).unwrap(),
        )?;

        for ping_record in records.ping {
            self.insert_record(
                measurement_id,
                "ping",
                serde_json::to_string(&ping_record).unwrap(),
            )?;
        }

        for download_record in records.download {
            self.insert_record(
                measurement_id,
                "download",
                serde_json::to_string(&download_record).unwrap(),
            )?;
        }

        for upload_record in records.upload {
            self.insert_record(
                measurement_id,
                "upload",
                serde_json::to_string(&upload_record).unwrap(),
            )?;
        }

        self.insert_record(
            measurement_id,
            "result",
            serde_json::to_string(&records.result).unwrap(),
        )?;

        Ok(())
    }

    fn insert_measurement(&self, start_record: &StartRecord) -> Result<u64, Error> {
        let mut stmt = self.connection.prepare("INSERT INTO measurements (timestamp, server_json) VALUES (?1, ?2) RETURNING measurement_id")?;
        let mut row_iter = stmt.query_map(
            params![
                start_record.timestamp,
                serde_json::to_string(&start_record.server).unwrap(),
            ],
            |row| row.get::<_, usize>(0),
        )?;

        let measurement_id = row_iter.next().expect("expected measurement_id")? as u64;
        Ok(measurement_id)
    }

    fn insert_record(
        &self,
        measurement_id: u64,
        record_type: &'static str,
        record_json: String,
    ) -> Result<(), Error> {
        self.connection.execute(
            "INSERT INTO records (measurement_id, type, details_json) VALUES (?1, ?2, ?3)",
            params![measurement_id, record_type, record_json],
        )?;

        Ok(())
    }
}

#[derive(Clone)]
pub struct DatabaseHandle {
    tx: Sender<Records>,
}

impl DatabaseHandle {
    pub fn insert_records(&self, records: Records) {
        self.tx.send(records).unwrap();
    }
}
