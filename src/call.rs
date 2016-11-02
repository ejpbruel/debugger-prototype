use convert::{FromJSValue, ToJSValue, UndefinedOr};
use exception::Result;
use js::jsapi;
use js::jsapi::{
    CallArgs,
    HandleValue,
    JSClass,
    JSClassOps,
    JSContext,
    JSFreeOp,
    JSObject,
    MutableHandleValue,
    Value
};
use std::os::raw::{c_char, c_uint};
use std::rc::Rc;

const JSCLASS_HAS_PRIVATE: c_uint = 1 << 0;

pub trait Call {
    unsafe fn call(&self, cx: *mut JSContext, argc: u32, vp: *mut Value) -> bool;
}

impl<T: ?Sized + Call> Call for Rc<T> {
    unsafe fn call(&self, cx: *mut JSContext, argc: u32, vp: *mut Value) -> bool {
        (**self).call(cx, argc, vp)
    }
}

impl FromJSValue for *mut Box<Call> {
    unsafe fn from_js_value(cx: *mut JSContext, v: HandleValue) -> Result<Self> {
        rooted!(in (cx) let obj = v.to_object());
        assert!(jsapi::JS_GetClass(obj.get()) == &CLASS);
        Ok(jsapi::JS_GetPrivate(obj.get()) as *mut Box<Call>)
    }
}

impl ToJSValue for *mut Box<Call> {
    unsafe fn to_js_value(&self, cx: *mut JSContext, rval: MutableHandleValue) -> bool {
        rooted!(in (cx) let obj = try_jsapi!(jsapi::JS_NewObject(cx, &CLASS)));
        jsapi::JS_SetPrivate(obj.get(), *self as _);
        obj.to_js_value(cx, rval)
    }
}

impl<T: 'static + Call + Clone> FromJSValue for T {
    unsafe fn from_js_value(cx: *mut JSContext, v: HandleValue) -> Result<Self> {
        <*mut Box<Call>>::from_js_value(cx, v).map(|call| {
            (**(call as *mut Box<T>)).clone()
        })
    }
}

impl<T: 'static + Call + Clone> ToJSValue for T {
    unsafe fn to_js_value(&self, cx: *mut JSContext, rval: MutableHandleValue) -> bool {
        Box::into_raw(Box::new(Box::new(self.clone()) as Box<Call>)).to_js_value(cx, rval)
    }
}

impl<T: 'static + Call + Clone> FromJSValue for Option<T> {
    unsafe fn from_js_value(cx: *mut JSContext, v: HandleValue) -> Result<Self> {
        UndefinedOr::<T>::from_js_value(cx, v).map(|call| call.into_option())
    }
}

impl<T: 'static + Call + Clone> ToJSValue for Option<T> {
    unsafe fn to_js_value(&self, cx: *mut JSContext, rval: MutableHandleValue) -> bool {
        match self {
            &Some(ref v) => v.to_js_value(cx, rval),
            &None => ().to_js_value(cx, rval)
        }
    }
}

static CLASS: JSClass = JSClass {
    name: b"call" as *const u8 as *const c_char,
    flags: JSCLASS_HAS_PRIVATE,
    cOps: &CLASS_OPS,
    reserved: [0 as *mut _; 3],
};

static CLASS_OPS: JSClassOps = JSClassOps {
    addProperty: None,
    delProperty: None,
    getProperty: None,
    setProperty: None,
    enumerate: None,
    resolve: None,
    mayResolve: None,
    finalize: Some(finalize),
    call: Some(call),
    hasInstance: None,
    construct: None,
    trace: None,
};

unsafe extern "C" fn call(cx: *mut JSContext, argc: u32, vp: *mut Value) -> bool {
    let args = CallArgs::from_vp(vp, argc);
    rooted!(in (cx) let callee = args.callee());
    (&*(jsapi::JS_GetPrivate(callee.get()) as *mut Box<Call>)).call(cx, argc, vp)
}

unsafe extern "C" fn finalize(_fop: *mut JSFreeOp, obj: *mut JSObject) {
    let _call = Box::from_raw(jsapi::JS_GetPrivate(obj) as *mut Box<Call>);
}
