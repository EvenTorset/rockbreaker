use rand::{distributions::Alphanumeric, Rng};

pub const TOKEN_HEADER: &str = "x-rockbreaker-token";

pub fn generate_host_token() -> String {
  rand::thread_rng()
    .sample_iter(Alphanumeric)
    .take(32)
    .map(char::from)
    .collect()
}

pub fn tokens_match(provided: &str, expected: &str) -> bool {
  let (a, b) = (provided.as_bytes(), expected.as_bytes());
  a.len() == b.len() && a.iter().zip(b).fold(0u8, |diff, (x, y)| diff | (x ^ y)) == 0
}
