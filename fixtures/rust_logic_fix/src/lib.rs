/// Returns a + b.
pub fn add(a: i32, b: i32) -> i32 {
    // BUG: wrong logic
    a - b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
        assert_eq!(add(10, -4), 6);
    }
}
