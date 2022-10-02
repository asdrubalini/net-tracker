use std::fmt::Debug;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::speedtest::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct StartRecord {
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PingDetails {
    jitter: f64,
    latency: f64,
    progress: Option<f64>,
    low: Option<f64>,
    high: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PingRecord {
    timestamp: Option<DateTime<Utc>>,

    ping: PingDetails,

    // used when processing before storing
    counter: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LatencyLoadedRecord {
    iqm: f64,
    low: Option<f64>,
    high: Option<f64>,
    jitter: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BandwidthDetails {
    bandwidth: u64,
    bytes: u64,
    elapsed: u64,
    progress: Option<f64>,
    latency: Option<LatencyLoadedRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadRecord {
    timestamp: Option<DateTime<Utc>>,

    download: BandwidthDetails,

    // used when processing before storing
    counter: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadRecord {
    timestamp: Option<DateTime<Utc>>,

    upload: BandwidthDetails,

    // used when processing before storing
    counter: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResultRecord {
    timestamp: DateTime<Utc>,

    ping: PingDetails,
    download: BandwidthDetails,
    upload: BandwidthDetails,

    #[serde(rename(deserialize = "packetLoss"))]
    packet_loss: u64,

    #[serde(rename(deserialize = "result"))]
    details: ResultDetails,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResultDetails {
    id: String,
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Record {
    #[serde(rename(deserialize = "testStart"))]
    Start(StartRecord),

    #[serde(rename(deserialize = "ping"))]
    Ping(PingRecord),

    #[serde(rename(deserialize = "download"))]
    Download(DownloadRecord),

    #[serde(rename(deserialize = "upload"))]
    Upload(UploadRecord),

    #[serde(rename(deserialize = "result"))]
    Result(ResultRecord),
}

impl Record {
    pub fn get_type(&self) -> &'static str {
        match self {
            Record::Start { .. } => "start",
            Record::Ping { .. } => "ping",
            Record::Download { .. } => "download",
            Record::Upload { .. } => "upload",
            Record::Result { .. } => "result",
        }
    }
}

impl TryFrom<&str> for Record {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let record = serde_json::from_str::<Record>(value)?;
        Ok(record)
    }
}

#[derive(Debug)]
pub struct Records {
    pub start: StartRecord,
    pub ping: Vec<PingRecord>,
    pub download: Vec<DownloadRecord>,
    pub upload: Vec<UploadRecord>,
    pub result: ResultRecord,
}

impl TryFrom<Vec<Record>> for Records {
    type Error = Error;

    fn try_from(records_vec: Vec<Record>) -> Result<Self, Self::Error> {
        let mut start: Option<StartRecord> = None;
        let mut ping: Vec<PingRecord> = vec![];
        let mut download: Vec<DownloadRecord> = vec![];
        let mut upload: Vec<UploadRecord> = vec![];
        let mut result: Option<ResultRecord> = None;

        for record in records_vec {
            match record {
                Record::Start(start_record) => start = Some(start_record),
                Record::Ping(ping_record) => ping.push(ping_record),
                Record::Download(download_record) => download.push(download_record),
                Record::Upload(upload_record) => upload.push(upload_record),
                Record::Result(result_record) => result = Some(result_record),
            }
        }

        if start.is_none() {
            println!("[speedtest] error: no start record");
            return Err(Error::InvalidRecordVec);
        }

        if result.is_none() {
            println!("[speedtest] error: no result record");
            return Err(Error::InvalidRecordVec);
        }

        Ok(Self {
            start: start.take().unwrap(),
            ping,
            download,
            upload,
            result: result.take().unwrap(),
        })
    }
}
