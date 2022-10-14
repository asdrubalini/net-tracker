use std::{
    thread::{self},
    time::{Duration, Instant},
};

use net_tracker::{database::DatabaseWorker, speedtest::Speedtest};

const SERVERS: [u32; 182] = [
    26941, 5762, 21160, 1324, 28390, 31495, 31501, 2811, 3009, 28727, 35148, 20680, 36716, 6820,
    23925, 37788, 1273, 2872, 10404, 37898, 2872, 10404, 37898, 25857, 12143, 34118, 34136, 32488,
    9071, 12143, 32488, 30583, 9329, 15920, 18695, 35124, 2657, 31496, 2567, 31498, 20372, 9070,
    17447, 7108, 6339, 9636, 2710, 22857, 3301, 19068, 20156, 38174, 31596, 2711, 29947, 30790,
    29262, 2275, 12799, 6483, 18054, 6483, 18054, 2755, 4620, 32003, 10633, 29953, 2864, 29953,
    10633, 5011, 10492, 21846, 4302, 1434, 11776, 3667, 11675, 3997, 8211, 26415, 24385, 20551,
    5502, 7839, 3667, 24385, 29875, 19177, 5502, 38731, 25609, 27363, 25146, 25258, 21966, 10450,
    26943, 21272, 31594, 33953, 34051, 24873, 34117, 38634, 38692, 5793, 6901, 31497, 6065, 3103,
    6065, 26670, 31610, 37785, 38619, 26010, 3231, 36728, 36930, 3384, 29948, 29949, 31500, 8651,
    9071, 25745, 31718, 29317, 24872, 33504, 38147, 3243, 7898, 9508, 11842, 395, 14671, 14761,
    16505, 20745, 5463, 8289, 7029, 31499, 6129, 7274, 7769, 29951, 10919, 17277, 36654, 37146,
    30357, 9967, 10406, 38096, 999, 31502, 5762, 11673, 37788, 11673, 20937, 5914, 37788, 20937,
    5914, 3998, 32391, 38037, 4826, 34132, 23323, 4826, 3998, 3679, 34133, 34135, 30460, 6484,
];

// const SPEEDTEST_SERVERS: [Speedtest; 7] = [
// // Vodafone Milano
// Speedtest::new(4302),
// // EOLO Milano
// Speedtest::new(11427),
// // Uania Milano
// Speedtest::new(33953),
// // WindTre Milano
// Speedtest::new(27363),
// // Fastweb Milano
// Speedtest::new(7839),
// // Fastweb Roma
// Speedtest::new(7898),
// // Vodafone Praga
// Speedtest::new(49678),
// ];

fn main() {
    // database
    let database_handle = {
        let (worker, handle) = DatabaseWorker::new().unwrap();
        thread::spawn(move || worker.run());
        handle
    };

    loop {
        let start = Instant::now();
        for server_id in SERVERS {
            let server = Speedtest::new(server_id);
            println!("starting test on server {server}");

            match server.measure() {
                Ok(records) => {
                    database_handle.insert_records(records);
                    thread::sleep(Duration::from_secs(10));
                }
                Err(err) => println!("got error from speedtest: {err}"),
            };
        }

        let elapsed = start.elapsed();
        println!("took {:?} for {} speedtests", elapsed, SERVERS.len());

        // let until_1h = Duration::from_secs(60 * 60).checked_sub(elapsed);

        // if let Some(sleep_dur) = until_1h {
        // thread::sleep(sleep_dur);
        // }
    }
}
