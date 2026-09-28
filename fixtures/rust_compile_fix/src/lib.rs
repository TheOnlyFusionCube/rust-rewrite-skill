/// Returns the double of n.
pub fn double(n: i32) -> i32 {
    // Deliberate type error: returning &str instead of i32
    "oops"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_double() {
        assert_eq!(double(21), 42);
    }
}
