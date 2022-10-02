use std::{thread, time::Duration};

use net_tracker::{database::DatabaseWorker, speedtest::Speedtest};

const SPEEDTEST_SERVER_ID: [u32; 2] = [4302, 11427];

fn main() {
    // database
    let database_handle = {
        let (worker, handle) = DatabaseWorker::new().unwrap();
        thread::spawn(move || worker.run());
        handle
    };

    loop {
        for server_id in SPEEDTEST_SERVER_ID {
            println!("starting test on server {server_id}");

            let server = Speedtest::new(server_id);
            let records = server.measure().unwrap();

            database_handle.insert_records(records);

            thread::sleep(Duration::from_secs(30));
        }

        thread::sleep(Duration::from_secs(10 * 60));
    }
}
