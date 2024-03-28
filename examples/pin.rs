use std::{marker::PhantomPinned, pin::Pin};

fn main() {
    heap_pinning();
}

fn heap_pinning() {
    let mut x = Box::pin(MaybeSelfRef::default());
    x.as_mut().init();
    println!("{}", x.as_ref().a);
    *x.as_mut().b().unwrap() = 2;
    println!("{}", x.as_ref().a);
}

#[derive(Default)]
struct Foo {
    a: MaybeSelfRef,
    b: String,
}

#[derive(Debug, Default)]
struct MaybeSelfRef {
    a: usize,
    b: Option<*mut usize>,
    _pin: PhantomPinned,
}

impl MaybeSelfRef {
    fn init(self: Pin<&mut Self>) {
        unsafe {
            let Self { a, b, .. } = self.get_unchecked_mut();
            *b = Some(a);
        }
    }

    fn b(self: Pin<&mut Self>) -> Option<&mut usize> {
        unsafe { self.get_unchecked_mut().b.map(|b| &mut *b) }
    }
}
