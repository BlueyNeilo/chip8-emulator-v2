use rand::Rng;

pub fn rng_byte() -> u8 {
    rand::thread_rng().gen_range(0,256) as u8
}
