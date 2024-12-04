use rand::Rng;

pub fn random_string(length: usize) -> String {
    let mut rng = rand::thread_rng();
    const CHARSET: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    (0..length)
        .map(|_| CHARSET[rng.gen_range(0..36)] as char)
        .collect()
}

pub fn random_int(min: i64, max: i64) -> i64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(min..max)
}
