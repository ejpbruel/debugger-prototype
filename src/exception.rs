use js::jsapi;
use js::jsapi::{JSContext, Value};
use js::jsval;
use rooted::Rooted;
use trace::TracedBox;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::result;

pub struct Exception(TracedBox<Value>);

impl Exception {
    pub unsafe fn from_pending_exception(cx: *mut JSContext) -> Exception {
        rooted!(in (cx) let mut v = jsval::UndefinedValue());
        assert!(jsapi::JS_GetPendingException(cx, v.handle_mut()));
        jsapi::JS_ClearPendingException(cx);
        Exception(TracedBox::new(cx, v.get()))
    }

    pub unsafe fn into_pending_exception(self, cx: *mut JSContext) -> bool {
        jsapi::JS_SetPendingException(cx, self.handle());
        false
    }
}

derive_rooted!(Value, Exception);

impl Debug for Exception {
    fn fmt(&self, _: &mut Formatter) -> fmt::Result {
        unimplemented!();
    }
}

pub type Result<T> = result::Result<T, Exception>;
