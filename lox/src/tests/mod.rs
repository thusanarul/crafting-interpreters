#[cfg(test)]
mod tests {
    use crate::run_file;

    #[test]
    fn test_fib() {
        run_file("src/tests/fibonacci").unwrap();
    }
}
