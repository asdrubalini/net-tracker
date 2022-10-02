use net_tracker::{database::Database, speedtest::Speedtest};

fn main() {
    let db = Database::new().unwrap();

    let ciao = Speedtest::new(11427);
    let records = ciao.measure().unwrap();

    db.insert_records(records).unwrap();
}
