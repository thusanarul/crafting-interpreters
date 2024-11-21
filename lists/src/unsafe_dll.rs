// Unsafe doubly-linked
use std::{marker::PhantomData, ptr::NonNull};

pub struct LinkedList<T> {
    front: Link<T>,
    back: Link<T>,
    len: usize,
    // Tells the compiler/others we are claiming our type behaves `as if` it stored a value T
    // Adding PhantomData makes that explicit.
    // Node does not have to do this, because it actually stores a T!
    _boo: PhantomData<T>,
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        while let Some(_) = self.pop_front() {}
    }
}

impl<T> LinkedList<T> {
    pub fn new() -> Self {
        Self {
            front: None,
            back: None,
            len: 0,
            _boo: PhantomData,
        }
    }

    pub fn push_front(&mut self, elem: T) {
        unsafe {
            // Creates the new element
            let new_node = Node {
                front: None,
                back: None,
                elem,
            };
            let new_front = NonNull::new_unchecked(Box::into_raw(Box::new(new_node)));

            if let Some(old_front) = self.front {
                // Non-empty list. Correct the references of the existing node.
                (*old_front.as_ptr()).front = Some(new_front);
                (*new_front.as_ptr()).back = Some(old_front);
            } else {
                // Empty list! set the .back of list to the new element
                // with some integrity tests
                self.back = Some(new_front)
            }
            self.front = Some(new_front);
            self.len += 1;
        }
    }
    pub fn pop_front(&mut self) -> Option<T> {
        unsafe {
            self.front.map(|node| {
                // Using Box::from_raw will drop the pointer when this block
                // goes out of scope.
                let boxed_node = Box::from_raw(node.as_ptr());
                let data = boxed_node.elem;

                // `back` is `next`. makes the front of the list the next node.
                self.front = boxed_node.back;
                if let Some(new_front) = self.front {
                    // cleans up the ref to the node that will go out of scope
                    (*new_front.as_ptr()).front = None;
                } else {
                    // If we end up here, front is now null!
                    // Therefore we have to correct the list and set the .back to None
                    self.back = None;
                }
                self.len -= 1;
                data
            })
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn front(&self) -> Option<&T> {
        unsafe { self.front.map(|node| &(*node.as_ptr()).elem) }
        // Can also write it like this with the `?` operator
        // unsafe { Some(&(*self.front?.as_ptr()).elem) }
    }

    pub fn front_mut(&mut self) -> Option<&mut T> {
        unsafe { self.front.map(|node| &mut (*node.as_ptr()).elem) }
    }
    pub fn iter(&self) -> Iter<T> {
        Iter {
            front: self.front,
            back: self.back,
            len: self.len,
            _boo: PhantomData,
        }
    }
}

// This will not work because *mut T is invariant over T
// type Link<T> = *mut Node<T>;

type Link<T> = Option<NonNull<Node<T>>>;

struct Node<T> {
    front: Link<T>,
    back: Link<T>,
    elem: T,
}

pub struct Iter<'a, T> {
    front: Link<T>,
    back: Link<T>,
    len: usize,
    _boo: PhantomData<&'a T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.front.map(|node| unsafe {
            self.len -= 1;
            // New front would be current front's next node
            self.front = (*node.as_ptr()).back;
            &(*node.as_ptr()).elem
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> IntoIterator for &'a LinkedList<T> {
    type Item = &'a T;

    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod test {
    use super::LinkedList;

    #[test]
    fn test_basic_front() {
        let mut list = LinkedList::new();

        // Try to break an empty list
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);

        // Try to break a one item list
        list.push_front(10);
        assert_eq!(list.len(), 1);
        assert_eq!(list.pop_front(), Some(10));
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);

        // Mess around
        list.push_front(10);
        assert_eq!(list.len(), 1);
        list.push_front(20);
        assert_eq!(list.len(), 2);
        list.push_front(30);
        assert_eq!(list.len(), 3);
        assert_eq!(list.pop_front(), Some(30));
        assert_eq!(list.len(), 2);
        list.push_front(40);
        assert_eq!(list.len(), 3);
        assert_eq!(list.pop_front(), Some(40));
        assert_eq!(list.len(), 2);
        assert_eq!(list.pop_front(), Some(20));
        assert_eq!(list.len(), 1);
        assert_eq!(list.pop_front(), Some(10));
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);
    }
}
