use js::jsapi::{HandleValueArray, Heap, Value};
use js::jsval;
use js::rust::GCMethods;
use std::cell::UnsafeCell;
use std::ptr;

pub trait HandleValueArrayExt: Sized {
    fn new() -> HandleValueArray;
    fn from_slice(values: &[Value]) -> HandleValueArray;
}

impl HandleValueArrayExt for HandleValueArray {
    fn new() -> HandleValueArray {
        HandleValueArray {
            length_: 0,
            elements_: ptr::null_mut(),
        }
    }

    fn from_slice(values: &[Value]) -> HandleValueArray {
        HandleValueArray {
            length_: values.len(),
            elements_: values.as_ptr()
        }
    }
}

pub trait HeapExt<T>
    where Self: Sized
{
    fn new(v: T) -> Self;
}

impl<T> HeapExt<*mut T> for Heap<*mut T>
    where *mut T: Copy + GCMethods<*mut T>
{
    fn new(ptr: *mut T) -> Heap<*mut T> {
        let mut result = Heap {
            ptr: UnsafeCell::new(ptr::null_mut()),
        };
        result.set(ptr);
        result
    }
}

impl HeapExt<Value> for Heap<Value>
    where Value: Copy + GCMethods<Value>
{
    fn new(ptr: Value) -> Heap<Value> {
        let mut result = Heap {
            ptr: UnsafeCell::new(jsval::UndefinedValue()),
        };
        result.set(ptr);
        result
    }
}
