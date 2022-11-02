# `lisp_iter` 
[![crates.io](https://img.shields.io/crates/v/lisp_iter)](https://crates.io/crates/lisp_iter/)

Single-pass no-alloc iterator for simple lisp or lisp-like expressions.

```rust
use lisp_iter::LispIter;

fn main() {
    let mut iter = LispIter::new(r#"(this-is-a-identifier :a 123 "wow") ; :a is shorthand for "a" "#);
    let mut list = iter.next().unwrap().into_iter(); // Retrieve first list in iterator

    println!("{:?}", list.next().unwrap()); // Identifier("this-is-a-identifier")
    println!("{:?}", list.next().unwrap()); // Quote("a")
    println!("{:?}", list.next().unwrap()); // Integer(0)
    println!("{:?}", list.next().unwrap()); // Quote("wow")
}
```

Useful to glance over anything lispy with minimal to 0 overhead.
