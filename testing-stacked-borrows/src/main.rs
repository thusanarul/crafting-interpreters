mod arrays;
mod interior_mut;
mod shared_ref;
mod testing_box;

fn first() {
    unsafe {
        let mut data = 10;
        let ref1 = &mut data;
        // let ref2 = &mut *ref1;
        let ptr2 = ref1 as *mut i32;

        // Wrong order
        // ptr2 will be popped of the borrow stack because ref1 is "below" and becomes live
        *ref1 += 1;
        *ptr2 += 2; // Will not exist in borrow stack at this point

        println!("{}", data);
    }
}

// &mut -> *mut -> &mut -> *mut
fn second() {
    unsafe {
        let mut data = 10;

        let ref1 = &mut data; // &mut
        let ptr2 = ref1 as *mut i32; //&mut -> *mut

        let ref3 = &mut *ptr2; // &mut -> *mut -> &mut
        let ptr4 = ref3 as *mut i32; // &mut -> *mut -> &mut -> *mut

        // Access the first raw pointer first
        *ptr2 += 2;

        // *ref1 += 1;

        // Then access things in the borrow stack order
        // This will fail for the same reason as in first()
        // Namely, the ptr2 is the live reborrow on the borrow stack. Newer borrows (ptr4 and ref3) are popped off.
        // Removing the use of ptr2 above will fix the issue.
        *ptr4 += 4;
        *ref3 += 3;
        *ptr2 += 2;
        *ref1 += 1;

        println!("{}", data);
    }
}

fn main() {
    // first();
    // second();
    // arrays::first();
    // arrays::second();
    // arrays::third();
    // shared_ref::first();
    // interior_mut::second();
    testing_box::first();
}
