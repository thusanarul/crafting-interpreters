fn opaque_read(value: &i32) {
    println!("{}", value);
}
pub fn first() {
    unsafe {
        let mut data = 10;

        let mref1 = &mut data;
        let ptr2 = mref1 as *mut i32;
        let sref3 = &*mref1;
        // cast shared ref to *const (which can only read)
        // but you are allowed to cast a *const to a *mut
        // let ptr4 = sref3 as *const i32 as *mut i32;

        // opaque_read(&*ptr4);
        *ptr2 += 2;
        opaque_read(sref3); // Reorder this so that it is not popped of the borrow stack
        *mref1 += 1;

        opaque_read(&data);
    }
}
