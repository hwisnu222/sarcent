use std::io::Cursor;

use murmur3::murmur3_32;

pub fn hash_to_coordinate(input: &str)-> i64{
    let mut cursor = Cursor::new(input.as_bytes());

    let hash_result = murmur3_32(&mut cursor, 0).unwrap_or(0);

    hash_result as i64
}
