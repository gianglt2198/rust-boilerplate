use nanoid::nanoid;

const ALPHABET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
const DEFAULT_SIZE: usize = 32;

pub fn generate_nanoid() -> String {
    let chars = ALPHABET.chars().collect::<Vec<char>>();
    nanoid!(DEFAULT_SIZE, &chars)
}
