use convert::{FromJSValue, Null, NullOr, ToJSValue};
use exception::Result;
use js::jsapi;
use js::jsapi::{HandleValue, JSContext, JSObject, MutableHandleValue};
use object::Object;
use std::ptr;
use utils;

pub enum Value {
    Undefined,
    Boolean(bool),
    Int32(i32),
    Double(f64),
    String(String),
    Object(Object),
    Null,
}

impl FromJSValue for Value {
    unsafe fn from_js_value(cx: *mut JSContext, v: HandleValue) -> Result<Self> {
        if v.is_undefined() {
            FromJSValue::from_js_value(cx, v).map(|()| Value::Undefined)
        } else if v.is_boolean() {
            FromJSValue::from_js_value(cx, v).map(|b| Value::Boolean(b))
        } else if v.is_int32() {
            FromJSValue::from_js_value(cx, v).map(|i| Value::Int32(i))
        } else if v.is_double() {
            FromJSValue::from_js_value(cx, v).map(|d| Value::Double(d))
        } else if v.is_string() {
            FromJSValue::from_js_value(cx, v).map(|s| Value::String(s))
        } else if v.is_object() {
            FromJSValue::from_js_value(cx, v).map(|o| Value::Object(o))
        } else if v.is_null() {
            FromJSValue::from_js_value(cx, v).map(|_: Null| Value::Null)
        } else {
            panic!()
        }
    }
}

impl ToJSValue for Value {
    unsafe fn to_js_value(&self, cx: *mut JSContext, rval: MutableHandleValue) -> bool {
        match self {
            &Value::Undefined => ().to_js_value(cx, rval),
            &Value::Boolean(b) => b.to_js_value(cx, rval),
            &Value::Int32(i) => i.to_js_value(cx, rval),
            &Value::Double(d) => d.to_js_value(cx, rval),
            &Value::String(ref s) => s.to_js_value(cx, rval),
            &Value::Object(ref o) => o.to_js_value(cx, rval),
            &Value::Null => Null.to_js_value(cx, rval),
        }
    }
}

pub enum CompletionValue {
    Return(Value),
    Throw(Value),
    Terminate,
}

impl FromJSValue for CompletionValue {
    unsafe fn from_js_value(cx: *mut JSContext, v: HandleValue) -> Result<Self> {
        NullOr::<*mut JSObject>::from_js_value(cx, v).and_then(|obj| {
            match obj.into_option() {
                Some(obj) => {
                    rooted!(in (cx) let obj = obj);
                    if try!(utils::has_property(cx, obj.handle(), "return")) {
                        assert!(!try!(utils::has_property(cx, obj.handle(), "throw")));
                        Ok(CompletionValue::Return(try!(utils::get_property(
                            cx,
                            obj.handle(),
                            "result"
                        ))))
                    } else {
                        assert!(try!(utils::has_property(cx, obj.handle(), "throw")));
                        Ok(CompletionValue::Throw(try!(utils::get_property(
                            cx,
                            obj.handle(),
                            "throw"
                        ))))

                    }
                },
                None => Ok(CompletionValue::Terminate)
            }
        })
    }
}

impl ToJSValue for CompletionValue {
    unsafe fn to_js_value(&self, cx: *mut JSContext, rval: MutableHandleValue) -> bool {
        match self {
            &CompletionValue::Return(ref value) => {
                rooted!(in (cx) let obj = try_jsapi!(jsapi::JS_NewObject(cx, ptr::null_mut())));
                try_jsapi!(utils::define_property(cx, obj.handle(), "result", value));
                obj.to_js_value(cx, rval)
            },
            &CompletionValue::Throw(ref value) => {
                rooted!(in (cx) let obj = try_jsapi!(jsapi::JS_NewObject(cx, ptr::null_mut())));
                try_jsapi!(utils::define_property(cx, obj.handle(), "throw", value));
                obj.to_js_value(cx, rval)
            },
            &CompletionValue::Terminate => {
                Null.to_js_value(cx, rval)
            }
        }
    }
}

pub type ResumptionValue = Option<CompletionValue>;

impl ToJSValue for ResumptionValue {
    unsafe fn to_js_value(&self, cx: *mut JSContext, rval: MutableHandleValue) -> bool {
        match self {
            &Some(ref value) => value.to_js_value(cx, rval),
            &None => ().to_js_value(cx, rval)
        }
    }
}
