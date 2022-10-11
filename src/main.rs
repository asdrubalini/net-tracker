use std::{
    thread::{self},
    time::{Duration, Instant},
};

use net_tracker::{database::DatabaseWorker, speedtest::Speedtest};

const SPEEDTEST_SERVERS: [Speedtest; 7] = [
    // Vodafone Milano
    Speedtest::new(4302),
    // EOLO Milano
    Speedtest::new(11427),
    // Uania Milano
    Speedtest::new(33953),
    // WindTre Milano
    Speedtest::new(27363),
    // Fastweb Milano
    Speedtest::new(7839),
    // Fastweb Roma
    Speedtest::new(7898),
    // Vodafone Praga
    Speedtest::new(49678),
];

fn main() {
    // database
    let database_handle = {
        let (worker, handle) = DatabaseWorker::new().unwrap();
        thread::spawn(move || worker.run());
        handle
    };

    loop {
        let start = Instant::now();
        for server in SPEEDTEST_SERVERS {
            println!("starting test on server {server}");

            match server.measure() {
                Ok(records) => database_handle.insert_records(records),
                Err(err) => println!("got error from speedtest: {err}"),
            };

            thread::sleep(Duration::from_secs(60));
        }

        let elapsed = start.elapsed();
        let until_1h = Duration::from_secs(60 * 60).checked_sub(elapsed);

        if let Some(sleep_dur) = until_1h {
            thread::sleep(sleep_dur);
        }
    }
}
