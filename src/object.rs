use convert::{FromJSValue, ToJSValue, UndefinedOr};
use ext::HandleValueArrayExt;
use exception::Result;
use js::jsapi;
use js::jsapi::{HandleValue, JSContext, JSObject, MutableHandleValue};
use rooted::Rooted;
use std::collections::BTreeMap;
use std::ptr;
use trace::TracedBox;
use utils;
use value::{CompletionValue, Value};

pub struct PropertyDescriptor {
    pub configurable: Option<bool>,
    pub enumerable: Option<bool>,
    pub writable: Option<bool>,
    pub value: Option<Value>,
    pub get: Option<Value>,
    pub set: Option<Value>
}

impl FromJSValue for PropertyDescriptor {
    unsafe fn from_js_value(cx: *mut JSContext, v: HandleValue) -> Result<Self> {
        rooted!(in (cx) let obj = v.to_object());
        Ok(PropertyDescriptor {
            configurable: if try!(utils::has_property(cx, obj.handle(), "configurable")) {
                Some(try!(utils::get_property(cx, obj.handle(), "configurable")))
            } else {
                None
            },
            enumerable: if try!(utils::has_property(cx, obj.handle(), "enumerable")) {
                Some(try!(utils::get_property(cx, obj.handle(), "enumerable")))
            } else {
                None
            },
            writable: if try!(utils::has_property(cx, obj.handle(), "writable")) {
                Some(try!(utils::get_property(cx, obj.handle(), "writable")))
            } else {
                None
            },
            value: if try!(utils::has_property(cx, obj.handle(), "value")) {
                Some(try!(utils::get_property(cx, obj.handle(), "value")))
            } else {
                None
            },
            get: if try!(utils::has_property(cx, obj.handle(), "get")) {
                Some(try!(utils::get_property(cx, obj.handle(), "get")))
            } else {
                None
            },
            set: if try!(utils::has_property(cx, obj.handle(), "set")) {
                Some(try!(utils::get_property(cx, obj.handle(), "set")))
            } else {
                None
            }
        })
    }
}

impl ToJSValue for PropertyDescriptor {
    unsafe fn to_js_value(&self, cx: *mut JSContext, rval: MutableHandleValue) -> bool {
        rooted!(in (cx) let obj = try_jsapi!(jsapi::JS_NewObject(cx, ptr::null_mut())));
        if let Some(ref configurable) = self.configurable {
            try_jsapi!(utils::define_property(cx, obj.handle(), "configurable", configurable));
        }
        if let Some(ref enumerable) = self.enumerable {
            try_jsapi!(utils::define_property(cx, obj.handle(), "enumerable", enumerable));
        }
        if let Some(ref writable) = self.writable {
            try_jsapi!(utils::define_property(cx, obj.handle(), "writable", writable));
        }
        if let Some(ref value) = self.value {
            try_jsapi!(utils::define_property(cx, obj.handle(), "value", value));
        }
        if let Some(ref get) = self.get {
            try_jsapi!(utils::define_property(cx, obj.handle(), "get", get));
        }
        if let Some(ref set) = self.set {
            try_jsapi!(utils::define_property(cx, obj.handle(), "set", set));
        }
        obj.to_js_value(cx, rval)
    }
}

pub enum PromiseState {
    Pending,
    Fulfilled,
    Rejected
}

impl FromJSValue for PromiseState {
    unsafe fn from_js_value(cx: *mut JSContext, v: HandleValue) -> Result<Self> {
        String::from_js_value(cx, v).map(|string| {
            if string == "pending" {
                PromiseState::Pending
            } else if string == "fulfilled" {
                PromiseState::Fulfilled
            } else if string == "rejected" {
                PromiseState::Rejected
            } else {
                panic!()
            }
        })
    }
}

pub struct Object(TracedBox<*mut JSObject>);

impl Object {
    pub fn new(cx: *mut JSContext, object: *mut JSObject) -> Object {
        Object(TracedBox::new(cx, object))
    }

    pub fn get_is_callable(&self, cx: *mut JSContext) -> Result<bool> {
        getter!(cx, self, "callable")
    }

    pub fn get_is_bound_function(&self, cx: *mut JSContext) -> Result<bool> {
        getter!(cx, self, "isBoundFunction")
    }

    pub fn get_is_proxy(&self, cx: *mut JSContext) -> Result<bool> {
        getter!(cx, self, "isProxy")
    }

    pub fn is_extensible(&self, cx: *mut JSContext) -> Result<bool> {
        method!(cx, self, "isExtensible")
    }

    pub fn is_sealed(&self, cx: *mut JSContext) -> Result<bool> {
        method!(cx, self, "isSealed")
    }

    pub fn is_frozen(&self, cx: *mut JSContext) -> Result<bool> {
        method!(cx, self, "isFrozen")
    }

    pub fn get_own_property_names(&self, cx: *mut JSContext) -> Result<Vec<String>> {
        method!(cx, self, "getOwnPropertyNames")
    }

    pub fn get_own_property_descriptor(
        &self,
        cx: *mut JSContext,
        name: &str
    ) -> Result<PropertyDescriptor> {
        method!(cx, self, "getOwnPropertyDescriptor", name)
    }

    pub fn prevent_extensions(&self, cx: *mut JSContext) -> Result<()> {
        method!(cx, self, "preventExtensions")
    }

    pub fn seal(&self, cx: *mut JSContext) -> Result<()> {
        method!(cx, self, "seal")
    }

    pub fn freeze(&self, cx: *mut JSContext) -> Result<()> {
        method!(cx, self, "freeze")
    }

    pub fn define_property(
        &self,
        cx: *mut JSContext,
        name: &str,
        descriptor: &PropertyDescriptor
    ) -> Result<()> {
        method!(cx, self, "defineProperty", name, descriptor)
    }

    pub fn define_properties(
        &self,
        cx: *mut JSContext,
        properties: BTreeMap<String, PropertyDescriptor>,
    ) -> Result<()> {
        method!(cx, self, "defineProperties", properties)
    }

    pub fn delete_property(&self, cx: *mut JSContext, name: &str) -> Result<bool> {
        method!(cx, self, "deleteProperty", name)
    }

    pub fn call(
        &self,
        cx: *mut JSContext,
        this: Value,
        arguments: &[Value]
    ) -> Result<CompletionValue> {
        method!(cx, self, "apply", this, arguments)
    }

    pub fn get_name(&self, cx: *mut JSContext) -> Result<Option<String>> {
        getter!(cx, self, "name").map(|name| {
            UndefinedOr::<String>::into_option(name)
        })
    }

    pub fn get_parameter_names(&self, cx: *mut JSContext) -> Result<Option<Vec<String>>> {
        getter!(cx, self, "parameterNames").map(|names| {
            UndefinedOr::<Vec<String>>::into_option(names)
        })
    }

    pub fn get_bound_target_function(&self, cx: *mut JSContext) -> Result<Option<Object>> {
        getter!(cx, self, "boundTargetFunction").map(|function| {
            UndefinedOr::<Object>::into_option(function)
        })
    }

    pub fn get_bound_this(&self, cx: *mut JSContext) -> Result<Option<Value>> {
        getter!(cx, self, "boundThis").map(|this| {
            UndefinedOr::<Value>::into_option(this)
        })
    }

    pub fn get_bound_arguments(&self, cx: *mut JSContext) -> Result<Option<Vec<Value>>> {
        getter!(cx, self, "boundArguments").map(|arguments| {
            UndefinedOr::<Vec<Value>>::into_option(arguments)
        })
    }

    pub fn get_proxy_target(&self, cx: *mut JSContext) -> Result<Option<Object>> {
        getter!(cx, self, "proxyTarget").map(|target| {
            UndefinedOr::<Object>::into_option(target)
        })
    }

    pub fn get_proxy_handler(&self, cx: *mut JSContext) -> Result<Option<Object>> {
        getter!(cx, self, "proxyHandler").map(|handler| {
            UndefinedOr::<Object>::into_option(handler)
        })
    }

    pub fn get_promise_state(&self, cx: *mut JSContext) -> Result<PromiseState> {
        getter!(cx, self, "promiseState")
    }

    pub fn get_promise_value(&self, cx: *mut JSContext) -> Result<Value> {
        getter!(cx, self, "promiseValue")
    }

    pub fn get_promise_reason(&self, cx: *mut JSContext) -> Result<Value> {
        getter!(cx, self, "promiseReason")
    }
}

derive_rooted!(*mut JSObject, Object);

derive_convert!(Object);
