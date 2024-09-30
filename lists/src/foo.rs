pub fn foo() -> Option<i32> {
    let mut bar = Some(0);

    while let Some(value) = bar {
        if value == 10 {
            break;
        } else {
            bar = Some(value + 1)
        }
    }

    return bar;
}

#[cfg(test)]
mod test {
    use crate::foo::foo;

    #[test]
    fn basic() {
        assert_eq!(Some(10), foo())
    }
}
