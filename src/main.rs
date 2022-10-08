use std::{thread, time::Duration};

use net_tracker::{database::DatabaseWorker, speedtest::Speedtest};

const SPEEDTEST_SERVERS: [Speedtest; 2] = [Speedtest::new(4302), Speedtest::new(11427)];

fn main() {
    // database
    let database_handle = {
        let (worker, handle) = DatabaseWorker::new().unwrap();
        thread::spawn(move || worker.run());
        handle
    };

    loop {
        for server in SPEEDTEST_SERVERS {
            println!("starting test on server {server}");

            match server.measure() {
                Ok(records) => database_handle.insert_records(records),
                Err(err) => println!("got error from speedtest: {err}"),
            };

            thread::sleep(Duration::from_secs(30));
        }

        thread::sleep(Duration::from_secs(5 * 60));
    }
}
