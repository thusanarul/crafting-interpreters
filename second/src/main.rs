use std::cell::RefCell;
use std::rc::Rc;

// type NodeLink = Option<Rc<RefCell<Node>>>;
type NodeLink = Option<Box<Node>>;

#[derive(Clone)]
struct Node {
    prev: NodeLink,
    next: NodeLink,
    value: i32,
}

impl Node {
    fn new(value: i32) -> Box<Node> {
        // Rc::new(RefCell::new(Node {
        //     value,
        //     prev: None,
        //     next: None,
        // }))

        Box::new(Node {
            value,
            prev: None,
            next: None,
        })
    }


}

struct LinkedList {
    first: NodeLink,
    last: NodeLink,
}

impl LinkedList {
    fn new() -> Self {
        LinkedList {first: None, last: None}
    }

   fn append(&mut self, value: i32) {
       let mut new_last = Node::new(value);

       // .take() moves the value of the Option, leaving a None in its place.
       match self.last.take() {
           None => {
               self.last = Some(new_last.clone())
           }
           Some(mut prev_last) => {
               prev_last.next = Some(new_last.clone());
               new_last.prev = Some(prev_last);

               self.last = Some(new_last)
           }
       }
   }
}



fn main() {
    println!("Hello, world!");
}
