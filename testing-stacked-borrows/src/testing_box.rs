pub fn first() {
    unsafe {
        let mut data = Box::new(10);
        let ptr1 = &mut *data as *mut i32;

        // Doing this out-of-order is Undefined Behaviour
        // *data += 10;
        // *ptr1 += 1;

        *ptr1 += 1;
        *data += 10;

        println!("{}", data);
    }
}
