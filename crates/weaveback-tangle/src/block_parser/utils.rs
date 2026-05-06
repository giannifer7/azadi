// weaveback-tangle/src/block_parser/utils.rs
// I'd Really Rather You Didn't edit this generated file.

/// Map of byte offset → 1-based line number.
pub(in crate::block_parser) fn build_line_table(source: &str) -> Vec<usize> {
    let mut table = Vec::with_capacity(source.len() + 1);
    let mut line = 1usize;
    for byte in source.bytes() {
        table.push(line);
        if byte == b'\n' {
            line += 1;
        }
    }
    table.push(line); // sentinel for end-of-file
    table
}

pub(in crate::block_parser) fn byte_to_line(table: &[usize], byte: usize) -> u32 {
    table.get(byte).copied().unwrap_or(1) as u32
}
