pub fn first() {
    unsafe {
        let mut data = [0; 10];
        let ref1_at_0 = &mut data[0]; // Reference to 0th element;
        let ptr2_at_0 = ref1_at_0 as *mut i32; // Pointer to 0th element;
        let ptr3_at_1 = ptr2_at_0.add(1); // Ptr to 1st element;

        *ptr3_at_1 += 3;
        *ptr2_at_0 += 2;
        *ref1_at_0 += 1;

        // Expecting [3, 3, 0, ...]
        println!("{:?}", &data[..])
    }
}

pub fn second() {
    unsafe {
        let mut data = [0; 10];

        // Slice tell the compiler:
        // "hey I'm taking a huge loan on all of the memory in my range"
        // so they know all the elements can be mutated.
        // But, why doesnt it then work in first() if ref1 takes a reference to a slice?
        let slice1 = &mut data[..];
        let (slice2_at_0, slice3_at_1) = slice1.split_at_mut(1);

        let ref4_at_0 = &mut slice2_at_0[0];
        let ref5_at_1 = &mut slice3_at_1[0];

        let ptr6_at_0 = ref4_at_0 as *mut i32;
        let ptr7_at_1 = ref5_at_1 as *mut i32;

        *ptr7_at_1 += 7;
        *ptr6_at_0 += 6;
        *ref5_at_1 += 5;
        *ref4_at_0 += 4;

        // Should be [10, 12, 0, ...]
        println!("{:?}", &data[..])
    }
}

pub fn third() {
    unsafe {
        let mut data = [0; 10];

        let slice_1_all = &mut data[..]; // Slice for the entire array

        // Turn a slice into a pointer
        let ptr2_all = slice_1_all.as_mut_ptr(); // Pointer for the entire array.

        let ptr3_at_0 = ptr2_all; // Pointer to 0th element
        let ptr4_at_1 = ptr2_all.add(1); // Pointer to 1th element
        let ref5_at_0 = &mut *ptr3_at_0; // Reference to 0th element;
        let ref6_at_1 = &mut *ptr4_at_1; // Reference to 1th element;

        *ref6_at_1 += 6;
        *ref5_at_0 += 5;
        *ptr4_at_1 += 4;
        *ptr3_at_0 += 3;

        for idx in 0..10 {
            *ptr2_all.add(idx) += idx;
        }

        for (idx, elem_ref) in slice_1_all.iter_mut().enumerate() {
            *elem_ref += idx;
        }

        println!("{:?}", &data[..])
    }
}
