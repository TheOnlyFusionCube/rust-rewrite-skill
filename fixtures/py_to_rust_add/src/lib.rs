/// Implement add so tests pass. Stub returns 0.
pub fn add(a: i32, b: i32) -> i32 {
    let _ = (a, b);
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
        assert_eq!(add(-1, 1), 0);
        assert_eq!(add(100, 23), 123);
    }
}
