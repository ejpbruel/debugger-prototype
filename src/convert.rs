use exception::{Exception, Result};
use js::{glue, jsapi};
use js::jsapi::{HandleValue, JSContext, JSObject, MutableHandleValue, JSString};
use js::jsval;
use std::collections::BTreeMap;
use std::slice;
use std::iter::FromIterator;
use std::ptr;
use utils;

pub trait FromJSValue: Sized {
    unsafe fn from_js_value(cx: *mut JSContext, v: HandleValue) -> Result<Self>;
}

impl FromJSValue for () {
    unsafe fn from_js_value(_cx: *mut JSContext, v: HandleValue) -> Result<Self> {
        assert!(v.is_undefined());
        Ok(())
    }
}


impl FromJSValue for bool {
    unsafe fn from_js_value(_cx: *mut JSContext, v: HandleValue) -> Result<Self> {
        Ok(v.to_boolean())
    }
}

impl FromJSValue for i32 {
    unsafe fn from_js_value(_cx: *mut JSContext, v: HandleValue) -> Result<Self> {
        if v.is_int32() {
            Ok(v.to_int32())
        } else {
            let mut i = 0;
            assert!(jsapi::JS_DoubleIsInt32(v.to_double(), &mut i));
            Ok(i)
        }
    }
}

impl FromJSValue for u32 {
    unsafe fn from_js_value(_cx: *mut JSContext, v: HandleValue) -> Result<Self> {
        if v.is_int32() {
            Ok(v.to_int32() as u32)
        } else {
            let d = v.to_double();
            assert!(d % 1.0 == 0.0 && u32::min_value() as f64 <= d &&
                d <= u32::max_value() as f64);
            Ok(d as u32)
        }
    }
}

impl FromJSValue for f64 {
    unsafe fn from_js_value(_cx: *mut JSContext, v: HandleValue) -> Result<Self> {
        Ok(v.to_number())
    }
}

impl FromJSValue for *mut JSString {
    unsafe fn from_js_value(_cx: *mut JSContext, v: HandleValue) -> Result<Self> {
        Ok(v.to_string())
    }
}

impl FromJSValue for *mut JSObject {
    unsafe fn from_js_value(_cx: *mut JSContext, v: HandleValue) -> Result<Self> {
        Ok(v.to_object())
    }
}

impl FromJSValue for String {
    unsafe fn from_js_value(cx: *mut JSContext, v: HandleValue) -> Result<Self> {
        FromJSValue::from_js_value(cx, v).and_then(|str| {
            if jsapi::JS_StringHasLatin1Chars(str) {
                let mut length = 0;
                Ok(String::from_iter(slice::from_raw_parts(
                    try_jsapi!(
                        cx,
                        jsapi::JS_GetLatin1StringCharsAndLength(cx, ptr::null(), str, &mut length
                    )),
                    length as usize
                ).iter().map(|&c| c as char)))
            } else {
                let mut length = 0;
                Ok(String::from_utf16_lossy(slice::from_raw_parts(
                    try_jsapi!(
                        cx,
                        jsapi::JS_GetTwoByteStringCharsAndLength(cx, ptr::null(), str, &mut length)
                    ),
                    length as usize
                )))
            }
        })
    }
}

impl<T: FromJSValue> FromJSValue for Vec<T> {
    unsafe fn from_js_value(cx: *mut JSContext, v: HandleValue) -> Result<Self> {
        rooted!(in (cx) let obj = v.to_object());
        let mut length = 0;
        if !jsapi::JS_GetArrayLength(cx, obj.handle(), &mut length) {
            return Err(Exception::from_pending_exception(cx));
        }
        (0..length).map(|index| {
            rooted!(in (cx) let mut element = jsval::UndefinedValue());
            if !jsapi::JS_GetElement(cx, obj.handle(), index, element.handle_mut()) {
                return Err(Exception::from_pending_exception(cx));
            }
            T::from_js_value(cx, element.handle())
        }).collect()
    }
}

pub trait ToJSValue {
    unsafe fn to_js_value(&self, cx: *mut JSContext, rval: MutableHandleValue) -> bool;
}

impl ToJSValue for () {
    unsafe fn to_js_value(&self, _cx: *mut JSContext, rval: MutableHandleValue) -> bool {
        rval.set(jsval::UndefinedValue());
        true
    }
}

impl ToJSValue for bool {
    unsafe fn to_js_value(&self, _cx: *mut JSContext, rval: MutableHandleValue) -> bool {
        rval.set(jsval::BooleanValue(*self));
        true
    }
}

impl ToJSValue for i32 {
    unsafe fn to_js_value(&self, _cx: *mut JSContext, rval: MutableHandleValue) -> bool {
        rval.set(jsval::Int32Value(*self));
        true
    }
}

impl ToJSValue for u32 {
    unsafe fn to_js_value(&self, cx: *mut JSContext, rval: MutableHandleValue) -> bool {
        if *self > 0x7FFFFFFF {
            (*self as f64).to_js_value(cx, rval)
        } else {
            (*self as i32).to_js_value(cx, rval)
        }
    }
}

impl ToJSValue for f64 {
    unsafe fn to_js_value(&self, _cx: *mut JSContext, rval: MutableHandleValue) -> bool {
        rval.set(glue::RUST_JS_NumberValue(*self));
        true
    }
}

impl ToJSValue for *mut JSString {
    unsafe fn to_js_value(&self, cx: *mut JSContext, rval: MutableHandleValue) -> bool {
        rval.set(jsval::StringValue(&**self));
        try_jsapi!(jsapi::JS_WrapValue(cx, rval));
        true
    }
}

impl ToJSValue for *mut JSObject {
    unsafe fn to_js_value(&self, cx: *mut JSContext, rval: MutableHandleValue) -> bool {
        rval.set(jsval::ObjectValue(&**self));
        try_jsapi!(jsapi::JS_WrapValue(cx, rval));
        true
    }
}

impl ToJSValue for str {
    unsafe fn to_js_value(&self, cx: *mut JSContext, rval: MutableHandleValue) -> bool {
        let chars = Vec::from_iter(self.encode_utf16());
        let str = try_jsapi!(jsapi::JS_NewUCStringCopyN(cx, chars.as_ptr(), chars.len()));
        try_jsapi!(str.to_js_value(cx, rval));
        true
    }
}

impl<'a, T: ToJSValue> ToJSValue for &'a [T] {
    unsafe fn to_js_value(&self, cx: *mut JSContext, rval: MutableHandleValue) -> bool {
        let length = self.len();
        rooted!(in (cx) let obj = try_jsapi!(jsapi::JS_NewArrayObject1(cx, length)));
        for index in 0..length {
            rooted!(in (cx) let mut v = jsval::UndefinedValue());
            try_jsapi!(self[index].to_js_value(cx, v.handle_mut()));
            try_jsapi!(jsapi::JS_SetElement(cx, obj.handle(), index as u32, v.handle()));
        }
        try_jsapi!(obj.to_js_value(cx, rval));
        true
    }
}

impl<K: ToString, V: ToJSValue> ToJSValue for BTreeMap<K, V> {
    unsafe fn to_js_value(&self, cx: *mut JSContext, rval: MutableHandleValue) -> bool {
        rooted!(in (cx) let obj = try_jsapi!(jsapi::JS_NewObject(cx, ptr::null_mut())));
        for (key, value) in self {
            try_jsapi!(utils::define_property(cx, obj.handle(), &key.to_string(), value));
        }
        try_jsapi!(obj.to_js_value(cx, rval));
        true
    }
}

pub struct Null;

impl FromJSValue for Null {
    unsafe fn from_js_value(_cx: *mut JSContext, v: HandleValue) -> Result<Self> {
        assert!(v.is_null());
        Ok(Null)
    }
}

impl ToJSValue for Null {
    unsafe fn to_js_value(&self, _cx: *mut JSContext, rval: MutableHandleValue) -> bool {
        rval.set(jsval::NullValue());
        true
    }
}

pub struct NullOr<T>(Option<T>);

impl<T> NullOr<T> {
    pub fn into_option(self) -> Option<T> {
        self.0
    }
}

impl<T: FromJSValue> FromJSValue for NullOr<T> {
    unsafe fn from_js_value(cx: *mut JSContext, v: HandleValue) -> Result<Self> {
        if v.is_null() {
            Ok(NullOr(None))
        } else {
            FromJSValue::from_js_value(cx, v).map(|value| NullOr(Some(value)))
        }
    }
}

impl<T: ToJSValue> ToJSValue for NullOr<T> {
    unsafe fn to_js_value(&self, cx: *mut JSContext, rval: MutableHandleValue) -> bool {
        match &self.0 {
            &Some(ref v) => v.to_js_value(cx, rval),
            &None => Null.to_js_value(cx, rval)
        }
    }
}

pub struct UndefinedOr<T>(Option<T>);

impl<T> UndefinedOr<T> {
    pub fn into_option(self) -> Option<T> {
        self.0
    }
}

impl<T: FromJSValue> FromJSValue for UndefinedOr<T> {
    unsafe fn from_js_value(cx: *mut JSContext, v: HandleValue) -> Result<Self> {
        if v.is_undefined() {
            Ok(UndefinedOr(None))
        } else {
            FromJSValue::from_js_value(cx, v).map(|value| UndefinedOr(Some(value)))
        }
    }
}

impl<T: ToJSValue> ToJSValue for UndefinedOr<T> {
    unsafe fn to_js_value(&self, cx: *mut JSContext, rval: MutableHandleValue) -> bool {
        match &self.0 {
            &Some(ref v) => v.to_js_value(cx, rval),
            &None => ().to_js_value(cx, rval)
        }
    }
}
