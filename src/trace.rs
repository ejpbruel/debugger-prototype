use ext::HeapExt;
use js::{glue, jsapi};
use js::jsapi::{
    Handle,
    Heap,
    JSContext,
    JSFunction,
    JSObject,
    JSScript,
    JSString,
    JSTracer,
    MutableHandle,
    Value,
    jsid,
};
use js::rust::GCMethods;
use rooted::Rooted;
use std::os::raw::c_void;

pub unsafe trait Trace {
    unsafe fn trace(&self, trc: *mut JSTracer);
}

unsafe impl Trace for Heap<*mut JSFunction> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        glue::CallFunctionTracer(trc, self as *const _ as *mut Self, c_str!("function"));
    }
}

unsafe impl Trace for Heap<*mut JSObject> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        glue::CallObjectTracer(trc, self as *const _ as *mut Self, c_str!("object"));
    }
}

unsafe impl Trace for Heap<*mut JSScript> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        glue::CallScriptTracer(trc, self as *const _ as *mut Self, c_str!("script"));
    }
}

unsafe impl Trace for Heap<*mut JSString> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        glue::CallStringTracer(trc, self as *const _ as *mut Self, c_str!("string"));
    }
}

unsafe impl Trace for Heap<Value> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        glue::CallValueTracer(trc, self as *const _ as *mut Self, c_str!("value"));
    }
}

unsafe impl Trace for Heap<jsid> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        glue::CallIdTracer(trc, self as *const _ as *mut Self, c_str!("id"));
    }
}

pub struct TracedBox<T: Copy + GCMethods<T>> where Heap<T>: HeapExt<T> + Trace {
    cx: *mut JSContext,
    ptr: Box<Heap<T>>
}

impl<T: Copy + GCMethods<T>> TracedBox<T> where Heap<T>: HeapExt<T> + Trace {
    pub fn new(cx: *mut JSContext, ptr: T) -> TracedBox<T> {
        let result = TracedBox {
            cx: cx,
            ptr: Box::new(Heap::new(ptr))
        };
        unsafe {
            assert!(jsapi::JS_AddExtraGCRootsTracer(
                jsapi::JS_GetRuntime(result.cx),
                Some(TracedBox::trace),
                result.ptr.as_ref() as *const Heap<T> as *mut _)
            );
        }
        result
    }

    unsafe extern "C" fn trace(trc: *mut JSTracer, data: *mut c_void) {
        (&*(data as *mut _ as *const Heap<T>)).trace(trc);
    }
}

impl<T: Copy + GCMethods<T>> Drop for TracedBox<T> where Heap<T>: HeapExt<T> + Trace {
    fn drop(&mut self) {
        unsafe {
            jsapi::JS_RemoveExtraGCRootsTracer(
                jsapi::JS_GetRuntime(self.cx),
                Some(TracedBox::trace),
                &*self.ptr as *const Heap<T> as *mut _
            )
        }
    }
}

impl<T: Copy + GCMethods<T>> Rooted<T> for TracedBox<T> where Heap<T>: HeapExt<T> + Trace {
    fn get(&self) -> T {
        unsafe { *self.ptr.ptr.get() }
    }

    fn handle(&self) -> Handle<T> {
        unsafe { Handle::from_marked_location(self.ptr.ptr.get()) }
    }

    fn handle_mut(&mut self) -> MutableHandle<T> {
        unsafe { MutableHandle::from_marked_location(self.ptr.ptr.get()) }
    }
}
