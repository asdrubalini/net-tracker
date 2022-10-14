use std::fmt::Debug;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::speedtest::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerDetails {
    pub id: u64,
    pub host: String,
    pub name: String,
    pub location: String,
    pub country: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StartRecord {
    pub timestamp: DateTime<Utc>,
    pub server: ServerDetails,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PingDetails {
    jitter: f64,
    latency: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    progress: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    low: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    high: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PingRecord {
    timestamp: DateTime<Utc>,

    ping: PingDetails,

    // used when processing before storing
    counter: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LatencyLoadedRecord {
    iqm: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    low: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    high: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    jitter: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BandwidthDetails {
    bandwidth: u64,
    bytes: u64,
    elapsed: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    progress: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    latency: Option<LatencyLoadedRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadRecord {
    timestamp: DateTime<Utc>,

    download: BandwidthDetails,

    // used when processing before storing
    counter: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadRecord {
    timestamp: DateTime<Utc>,

    upload: BandwidthDetails,

    // used when processing before storing
    counter: Option<u64>,
}

#[derive(Serialize, Deserialize)]
pub struct ResultRecord {
    timestamp: DateTime<Utc>,

    ping: PingDetails,
    download: BandwidthDetails,
    upload: BandwidthDetails,

    #[serde(rename(deserialize = "packetLoss"))]
    packet_loss: Option<u64>,

    #[serde(rename(deserialize = "result"))]
    details: ResultDetails,
}

impl Debug for ResultRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "date={:?}, ping={:.2} ms, download={:.2} Mbps, upload={:.2} Mbps",
            self.timestamp,
            self.ping.latency,
            self.download.bandwidth as f64 / 125000.0,
            self.upload.bandwidth as f64 / 125000.0
        )
    }
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

        // sort records, remove progress % and set counters
        ping.sort_by(|a, b| a.ping.progress.partial_cmp(&b.ping.progress).unwrap());

        for (i, elem) in ping.iter_mut().enumerate() {
            elem.counter = Some(i as u64);
            elem.ping.progress = None;
        }

        download.sort_by(|a, b| {
            a.download
                .progress
                .partial_cmp(&b.download.progress)
                .unwrap()
        });

        for (i, elem) in download.iter_mut().enumerate() {
            elem.counter = Some(i as u64);
            elem.download.progress = None;
        }

        upload.sort_by(|a, b| a.upload.progress.partial_cmp(&b.upload.progress).unwrap());

        for (i, elem) in upload.iter_mut().enumerate() {
            elem.counter = Some(i as u64);
            elem.upload.progress = None;
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
