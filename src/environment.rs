use convert::{FromJSValue, NullOr, ToJSValue};
use exception::Result;
use ext::HandleValueArrayExt;
use js::jsapi::{HandleValue, JSContext, JSObject};
use object::Object;
use rooted::Rooted;
use trace::TracedBox;
use utils;
use value::Value;

pub enum EnvironmentType {
    Declarative,
    Object,
    With
}

impl FromJSValue for EnvironmentType {
    unsafe fn from_js_value(cx: *mut JSContext, v: HandleValue) -> Result<Self> {
        String::from_js_value(cx, v).map(|string| {
            if string == "declarative" {
                EnvironmentType::Declarative
            } else if string == "object" {
                EnvironmentType::Object
            } else if string == "with" {
                EnvironmentType::With
            } else {
                panic!()
            }
        })
    }
}

struct OptimizedOutOrValue(Option<Value>);

impl OptimizedOutOrValue {
    pub fn into_option(self) -> Option<Value> {
        self.0
    }
}

impl FromJSValue for OptimizedOutOrValue {
    unsafe fn from_js_value(cx: *mut JSContext, v: HandleValue) -> Result<Self> {
        if v.is_object() {
            rooted!(in (cx) let obj = v.to_object());
            try!(utils::has_property(cx, obj.handle(), "optimizedOut"));
            if try!(utils::get_property(cx, obj.handle(), "optimizedOut")) {
                return Ok(OptimizedOutOrValue(None));
            }
        }
        FromJSValue::from_js_value(cx, v).map(|value| OptimizedOutOrValue(Some(value)))
    }
}

pub struct Environment(TracedBox<*mut JSObject>);

impl Environment {
    pub fn new(cx: *mut JSContext, environment: *mut JSObject) -> Environment {
        Environment(TracedBox::new(cx, environment))
    }

    pub fn get_is_inspectable(&self, cx: *mut JSContext) -> Result<bool> {
        getter!(cx, self, "inspectable")
    }

    pub fn get_is_optimized_out(&self, cx: *mut JSContext) -> Result<bool> {
        getter!(cx, self, "optimizedOut")
    }

    pub fn get_parent(&self, cx: *mut JSContext) -> Result<Option<Environment>> {
        getter!(cx, self, "parent").map(|environment| {
            NullOr::<Environment>::into_option(environment)
        })
    }

    pub fn get_object(&self, cx: *mut JSContext) -> Result<Object> {
        getter!(cx, self, "object")
    }

    pub fn names(&self, cx: *mut JSContext) -> Result<Vec<String>> {
        method!(cx, self, "names")
    }

    pub fn get_variable(&self, cx: *mut JSContext, name: &str) -> Result<Option<Value>> {
        method!(cx, self, "getVariable", name).map(|value| {
            OptimizedOutOrValue::into_option(value)
        })
    }

    pub fn set_variable(&self, cx: *mut JSContext, name: &str, value: &Value) -> Result<()> {
        method!(cx, self, "setVariable", name, value)
    }
}

derive_rooted!(*mut JSObject, Environment);

derive_convert!(Environment);
