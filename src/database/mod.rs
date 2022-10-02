use std::path::Path;

use rusqlite::{params, Connection, Error};

use crate::speedtest::types::{Records, StartRecord};

pub struct Database(Connection);

impl Database {
    pub fn new() -> Result<Self, Error> {
        let path = Path::new("./results.db");
        let needs_init = !path.exists();

        let connection = Connection::open("./results.db")?;

        if needs_init {
            println!("[database] importing schema...");
            connection.execute_batch(include_str!("./schema.sql"))?;
        }

        Ok(Self(connection))
    }

    pub fn insert_records(&self, records: Records) -> Result<(), Error> {
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
        let mut stmt = self.0.prepare("INSERT INTO measurements (timestamp, server_json) VALUES (?1, ?2) RETURNING measurement_id")?;
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
        self.0.execute(
            "INSERT INTO records (measurement_id, type, details_json) VALUES (?1, ?2, ?3)",
            params![measurement_id, record_type, record_json],
        )?;

        Ok(())
    }
}
