use std::sync::Arc; 
use std::ops::Deref; 

pub unsafe trait SafeDeref: Deref {}
unsafe impl<'a, T: ?Sized> SafeDeref for &'a T {
}
unsafe impl<T: ?Sized> SafeDeref for Arc<T> {
}
unsafe impl<T: ?Sized> SafeDeref for Box<T> {
}

pub enum OwnedOrRef<T: 'static> {
    Owned(T),
    Ref(&'static T),
}

impl<T> Deref for OwnedOrRef<T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        match *self {
            OwnedOrRef::Owned(ref v) => v,
            OwnedOrRef::Ref(v) => v,
        }
    }
}
