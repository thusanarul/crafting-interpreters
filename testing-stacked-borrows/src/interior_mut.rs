use std::cell::{Cell, UnsafeCell};

pub fn first() {
    unsafe {
        let mut data = Cell::new(10);
        let mref1 = &mut data;
        let ptr2 = mref1 as *mut Cell<i32>;
        let sref3 = &*mref1;

        sref3.set(sref3.get() + 3);
        (*ptr2).set((*ptr2).get() + 2);
        mref1.set(mref1.get() + 1);

        println!("{}", data.get())
    }
}

fn opaque_read(val: &i32) {
    println!("{}", val)
}

pub fn second() {
    unsafe {
        let mut data = UnsafeCell::new(10);
        // let mref1 = data.get_mut();
        // Gets mut ref to outside, and not inside like above
        let mref1 = &mut data;
        // let ptr2 = mref1 as *mut i32;
        // Gets a raw pointer to the inside
        let ptr2 = mref1.get();
        // let sref3 = &*ptr2;
        // Gets a sharef ref to the outside
        let sref3 = &*mref1;

        *ptr2 += 2;
        opaque_read(&*sref3.get());
        *sref3.get() += 3;
        *mref1.get() += 1;

        println!("{:?}", data.get())
    }
}

pub fn third() {
    unsafe {
        let mut data = UnsafeCell::new(10);
        let mref1 = &mut data;

        // Unlike second(), we first grab a shared ref to the outside.
        let sref2 = &*mref1;
        // Then we grab the pointer from the shared ref to be *safe*(?)
        let ptr3 = sref2.get();

        *ptr3 += 3;
        opaque_read(&*sref2.get());
        *sref2.get() += 2;
        *mref1.get() += 1;

        println!("{}", *data.get())
    }
}
