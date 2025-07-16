pub fn simple_test() { println!("Simple test works!"); }
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_test() {
        simple_test();
    }
}
