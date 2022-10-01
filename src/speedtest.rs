use std::{
    alloc::LayoutErr,
    collections::HashMap,
    io::{self, BufRead, BufReader},
    process::{Command, Stdio},
};

use chrono::{DateTime, Utc};
use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Input/Output error: {0:#?}")]
    InputOutput(#[from] io::Error),
    #[error("JSON decode error: {0:#?}")]
    JsonDecode(#[from] serde_json::Error),
    #[error("JSON missing key: {0}")]
    JsonMissingKey(&'static str),
    #[error("JSON invalid value: {0}")]
    JsonInvalidValue(&'static str),
}

#[derive(Debug)]
enum Record {
    Start {
        timestamp: DateTime<Utc>,
    },

    Ping {
        timestamp: DateTime<Utc>,
        jitter: f64,
        latency: f64,
        progress: f64,
    },

    Download {
        timestamp: DateTime<Utc>,
        bandwidth: u64,
        bytes_total: u64,
        elapsed: u64,
        iqm_latency: Option<f64>,
        progress: f64,
    },

    Upload {
        timestamp: DateTime<Utc>,
        bandwidth: u64,
        bytes_total: u64,
        elapsed: u64,
        iqm_latency: Option<f64>,
        progress: f64,
    },

    Result {
        timestamp: DateTime<Utc>,

        ping_jitter: f64,
        ping_avg: f64,
        ping_min: f64,
        ping_max: f64,

        download_bandwidth: u64,
        download_bytes: u64,
        download_elapsed: u64,
        download_latency_iqm: f64,
        download_latency_low: f64,
        download_latency_high: f64,
        download_latency_jitter: f64,

        upload_bandwidth: u64,
        upload_bytes: u64,
        upload_elapsed: u64,
        upload_latency_iqm: f64,
        upload_latency_low: f64,
        upload_latency_high: f64,
        upload_latency_jitter: f64,

        packet_loss: u64,

        result_uuid: String,
        result_url: String,
    },
}

impl TryFrom<&str> for Record {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let record_json = serde_json::from_str::<HashMap<String, Value>>(value)?;

        // this code sucks but it was not feasible to make it work with serde macros

        let result_type = record_json
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or(Error::JsonMissingKey("type"))?;

        let timestamp = {
            let timestamp_str = record_json
                .get("timestamp")
                .and_then(|v| v.as_str())
                .ok_or(Error::JsonMissingKey("timestamp"))?;

            timestamp_str
                .parse::<DateTime<Utc>>()
                .map_err(|_| Error::JsonInvalidValue("timestamp"))?
        };

        let record = match result_type {
            "testStart" => Record::Start { timestamp },

            "ping" => {
                let ping = record_json
                    .get("ping")
                    .ok_or(Error::JsonInvalidValue("ping"))?;

                Record::Ping {
                    timestamp,
                    jitter: ping
                        .get("jitter")
                        .and_then(|v| v.as_f64())
                        .ok_or(Error::JsonInvalidValue("jitter"))?,
                    latency: ping
                        .get("latency")
                        .and_then(|v| v.as_f64())
                        .ok_or(Error::JsonInvalidValue("latency"))?,
                    progress: ping
                        .get("progress")
                        .and_then(|v| v.as_f64())
                        .ok_or(Error::JsonInvalidValue("progress"))?,
                }
            }

            "download" | "upload" => {
                let data = record_json
                    .get(result_type)
                    .ok_or(Error::JsonInvalidValue("download or upload"))?;

                let bandwidth = data
                    .get("bandwidth")
                    .and_then(|v| v.as_u64())
                    .ok_or(Error::JsonInvalidValue("bandwidth"))?;

                let bytes_total = data
                    .get("bytes")
                    .and_then(|v| v.as_u64())
                    .ok_or(Error::JsonInvalidValue("bytes"))?;

                let elapsed = data
                    .get("elapsed")
                    .and_then(|v| v.as_u64())
                    .ok_or(Error::JsonInvalidValue("elapsed"))?;

                let iqm_latency = data
                    .get("latency")
                    .and_then(|latency| latency.get("iqm").and_then(|iqm| iqm.as_f64()));

                let progress = data
                    .get("progress")
                    .and_then(|v| v.as_f64())
                    .ok_or(Error::JsonInvalidValue("elapsed"))?;

                match result_type {
                    "download" => Record::Download {
                        timestamp,
                        bandwidth,
                        bytes_total,
                        elapsed,
                        iqm_latency,
                        progress,
                    },

                    "upload" => Record::Upload {
                        timestamp,
                        bandwidth,
                        bytes_total,
                        elapsed,
                        iqm_latency,
                        progress,
                    },

                    _ => unreachable!(),
                }
            }

            "result" => {
                let ping = record_json
                    .get("ping")
                    .ok_or(Error::JsonInvalidValue("ping"))?;

                let download = record_json
                    .get("download")
                    .ok_or(Error::JsonInvalidValue("download"))?;

                let download_latency = download
                    .get("latency")
                    .ok_or(Error::JsonInvalidValue("latency"))?;

                let upload = record_json
                    .get("upload")
                    .ok_or(Error::JsonInvalidValue("upload"))?;

                let upload_latency = download
                    .get("latency")
                    .ok_or(Error::JsonInvalidValue("latency"))?;

                Record::Result {
                    timestamp,

                    ping_jitter: ping
                        .get("jitter")
                        .and_then(|v| v.as_f64())
                        .ok_or(Error::JsonInvalidValue("jitter"))?,
                    ping_avg: ping
                        .get("latency")
                        .and_then(|v| v.as_f64())
                        .ok_or(Error::JsonInvalidValue("latency"))?,
                    ping_min: ping
                        .get("low")
                        .and_then(|v| v.as_f64())
                        .ok_or(Error::JsonInvalidValue("low"))?,
                    ping_max: ping
                        .get("high")
                        .and_then(|v| v.as_f64())
                        .ok_or(Error::JsonInvalidValue("high"))?,

                    download_bandwidth: download
                        .get("bandwidth")
                        .and_then(|v| v.as_u64())
                        .ok_or(Error::JsonInvalidValue("bandwidth"))?,
                    download_bytes: download
                        .get("bytes")
                        .and_then(|v| v.as_u64())
                        .ok_or(Error::JsonInvalidValue("bytes"))?,
                    download_elapsed: download
                        .get("elapsed")
                        .and_then(|v| v.as_u64())
                        .ok_or(Error::JsonInvalidValue("elapsed"))?,
                    download_latency_iqm: (),
                    download_latency_low: (),
                    download_latency_high: (),
                    download_latency_jitter: (),

                    upload_bandwidth: (),
                    upload_bytes: (),
                    upload_elapsed: (),
                    upload_latency_iqm: (),
                    upload_latency_low: (),
                    upload_latency_high: (),
                    upload_latency_jitter: (),

                    packet_loss: (),
                    result_uuid: (),
                    result_url: (),
                }
            }

            _ => return Err(Error::JsonInvalidValue("type")),
        };

        Ok(record)
    }
}

pub struct Speedtest {
    server_id: u32,
}

impl Speedtest {
    pub fn new(server_id: u32) -> Self {
        Self { server_id }
    }

    pub fn start(self) -> Result<(), Error> {
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

        reader
            .lines()
            .filter_map(|line| line.ok())
            .for_each(|line| {
                let ciao = Record::try_from(line.as_str()).unwrap();
                println!("{:#?}", ciao);
            });
        Ok(())
    }
}
