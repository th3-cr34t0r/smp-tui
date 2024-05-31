use rand::*;

pub fn get_network_hashrate() -> Vec<(f64, f64)> {
    let mut data: Vec<(f64, f64)> = vec![];
    let mut rng = rand::thread_rng();
    for i in 0..10 {
        data.insert(i, ((i + 1) as f64, rng.gen_range(15..=20) as f64));
    }
    data
}
