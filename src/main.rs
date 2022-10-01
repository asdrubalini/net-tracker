use net_tracker::speedtest::Speedtest;

fn main() {
    let ciao = Speedtest::new(11427);
    ciao.start().unwrap();
}
