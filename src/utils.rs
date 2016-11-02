use convert::{FromJSValue, ToJSValue};
use exception::Result;
use js::{JSCLASS_GLOBAL_SLOT_COUNT, JSCLASS_IS_GLOBAL, JSCLASS_RESERVED_SLOTS_MASK, jsapi};
use js::jsapi::{
    CompartmentOptions,
    HandleObject,
    HandleValueArray,
    JSCLASS_RESERVED_SLOTS_SHIFT,
    JSClass,
    JSClassOps,
    JSContext,
    MutableHandleObject,
    OnNewGlobalHookOption,
};
use js::jsval;
use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;

pub unsafe fn new_global_object(cx: *mut JSContext, rval: MutableHandleObject) -> bool {
    static CLASS: JSClass = JSClass {
        name: b"global" as *const u8 as *const c_char,
        flags: JSCLASS_IS_GLOBAL |
            (JSCLASS_GLOBAL_SLOT_COUNT & JSCLASS_RESERVED_SLOTS_MASK) <<
                JSCLASS_RESERVED_SLOTS_SHIFT,
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
        finalize: None,
        call: None,
        hasInstance: None,
        construct: None,
        trace: Some(jsapi::JS_GlobalObjectTraceHook),
    };

    rooted!(in (cx) let global = jsapi::JS_NewGlobalObject(
        cx,
        &CLASS,
        ptr::null_mut(),
        OnNewGlobalHookOption::FireOnNewGlobalHook,
        &CompartmentOptions::default()
    ));
    if global.get().is_null() {
        return false;
    }
    rval.set(global.get());
    true
}

pub unsafe fn define_property<T: ToJSValue>(
    cx: *mut JSContext,
    obj: HandleObject,
    name: &str,
    value: &T
) -> bool {
    rooted!(in (cx) let mut v = jsval::UndefinedValue());
    !value.to_js_value(cx, v.handle_mut()) || jsapi::JS_DefineProperty(
        cx,
        obj,
        CString::new(name).unwrap().as_ptr(),
        v.handle(),
        0,
        None,
        None
    )
}

pub unsafe fn has_property(cx: *mut JSContext, obj: HandleObject, name: &str) -> Result<bool> {
    let mut found = false;
    try_jsapi!(cx, jsapi::JS_HasProperty(cx, obj, CString::new(name).unwrap().as_ptr(), &mut found));
    Ok(found)
}

pub unsafe fn get_property<T: FromJSValue>(
    cx: *mut JSContext,
    obj: HandleObject,
    name: &str
) -> Result<T> {
    rooted!(in (cx) let mut rval = jsval::UndefinedValue());
    try_jsapi!(cx, jsapi::JS_GetProperty(
        cx,
        obj,
        CString::new(name).unwrap().as_ptr(),
        rval.handle_mut()
    ));
    FromJSValue::from_js_value(cx, rval.handle())
}

pub unsafe fn set_property<T: ToJSValue>(
    cx: *mut JSContext,
    obj: HandleObject,
    name: &str,
    value: T
) -> bool {
    rooted!(in (cx) let mut v = jsval::UndefinedValue());
    try_jsapi!(value.to_js_value(cx, v.handle_mut()));
    try_jsapi!(jsapi::JS_SetProperty(
        cx,
        obj,
        CString::new(name).unwrap().as_ptr(),
        v.handle()
    ));
    true
}

pub unsafe fn get_element<T: FromJSValue>(
    cx: *mut JSContext,
    obj: HandleObject,
    index: usize
) -> Result<T> {
    rooted!(in (cx) let mut rval = jsval::UndefinedValue());
    try_jsapi!(cx, jsapi::JS_GetElement(
        cx,
        obj,
        index as u32,
        rval.handle_mut()
    ));
    FromJSValue::from_js_value(cx, rval.handle())
}

pub unsafe fn set_element<T: ToJSValue>(
    cx: *mut JSContext,
    obj: HandleObject,
    index: usize,
    value: &T
) -> bool {
    rooted!(in (cx) let mut v = jsval::UndefinedValue());
    try_jsapi!(value.to_js_value(cx, v.handle_mut()));
    try_jsapi!(jsapi::JS_SetElement(
        cx,
        obj,
        index as u32,
        v.handle()
    ));
    true
}

pub unsafe fn call_method<T: FromJSValue>(
    cx: *mut JSContext,
    obj: HandleObject,
    name: &str,
    args: &HandleValueArray
) -> Result<T> {
    rooted!(in (cx) let mut rval = jsval::UndefinedValue());
    try_jsapi!(cx, jsapi::JS_CallFunctionName(
        cx,
        obj,
        CString::new(name).unwrap().as_ptr(),
        args,
        rval.handle_mut()
    ));
    FromJSValue::from_js_value(cx, rval.handle())
}
