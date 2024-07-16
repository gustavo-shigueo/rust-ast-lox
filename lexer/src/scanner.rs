#[derive(Debug, Default)]
pub struct Scanner {
    source: String,
    current: u32,
    lexeme_start: u32,
    tokens: Vec<()>,
}
