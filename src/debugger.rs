use call::Call;
use convert::{FromJSValue, ToJSValue};
use exception::Result;
use ext::HandleValueArrayExt;
use frame::Frame;
use js::jsapi;
use js::jsapi::{
    CallArgs,
    HandleObject,
    HandleValueArray,
    JSContext,
    JSObject,
    Value,
};
use js::jsval;
use object::Object;
use rooted::Rooted;
use script::Script;
use std::ptr;
use std::rc::Rc;
use trace::TracedBox;
use utils;
use value::ResumptionValue;

pub trait OnNewScript {
    fn on_new_script(&self, cx: *mut JSContext, script: &Script) -> Result<()>;
}

impl Call for OnNewScript {
    unsafe fn call(&self, cx: *mut JSContext, argc: u32, vp: *mut Value) -> bool {
        let args = CallArgs::from_vp(vp, argc);
        match self.on_new_script(cx, &Script::from_js_value(cx, args.get(0)).unwrap()) {
            Ok(result) => result.to_js_value(cx, args.rval()),
            Err(exception) => exception.into_pending_exception(cx)
        }
    }
}

pub trait OnDebuggerStatement {
    fn on_debugger_statement(&self, cx: *mut JSContext, frame: &Frame) -> Result<ResumptionValue>;
}

impl Call for OnDebuggerStatement {
    unsafe fn call(&self, cx: *mut JSContext, argc: u32, vp: *mut Value) -> bool {
        let args = CallArgs::from_vp(vp, argc);
        match self.on_debugger_statement(cx, &Frame::from_js_value(cx, args.get(0)).unwrap()) {
            Ok(result) => result.to_js_value(cx, args.rval()),
            Err(exception) => exception.into_pending_exception(cx)
        }
    }
}

pub struct Debugger(TracedBox<*mut JSObject>);

impl Debugger {
    pub fn new(cx: *mut JSContext) -> Debugger {
        unsafe {
            rooted!(in (cx) let mut global = ptr::null_mut());
            assert!(utils::new_global_object(cx, global.handle_mut()));
            let _ac = ::js::jsapi::JSAutoCompartment::new(cx, global.get());
            assert!(jsapi::JS_DefineDebuggerObject(cx, global.handle()));
            rooted!(in (cx) let mut value = jsval::UndefinedValue());
            assert!(jsapi::JS_GetProperty(
                cx,
                global.handle(),
                c_str!("Debugger"),
                value.handle_mut()
            ));
            rooted!(in (cx) let mut debugger = ptr::null_mut());
            assert!(jsapi::Construct1(
                cx,
                value.handle(),
                &HandleValueArray::new(),
                debugger.handle_mut()
            ));
            Debugger(TracedBox::new(cx, debugger.get()))
        }
    }

    pub fn has_debuggee(&self, cx: *mut JSContext, global: HandleObject) -> Result<bool> {
        method!(cx, self, "hasDebuggee", global)
    }

    pub fn get_debuggees(&self, cx: *mut JSContext) -> Result<Vec<Object>> {
        method!(cx, self, "getDebuggees")
    }

    pub fn add_debuggee(&self, cx: *mut JSContext, global: HandleObject) -> Result<Object> {
        method!(cx, self, "addDebuggee", global)
    }

    pub fn add_all_globals_as_debuggees(&self, cx: *mut JSContext) -> Result<()> {
        method!(cx, self, "addAllGlobalsAsDebuggees")
    }

    pub fn remove_debuggee(&self, cx: *mut JSContext, global: HandleObject) -> Result<()> {
        method!(cx, self, "removeDebuggee", global)
    }

    pub fn remove_all_debuggees(&self, cx: *mut JSContext) -> Result<()> {
        method!(cx, self, "removeAllDebuggees")
    }

    pub fn get_on_debugger_statement(
        &self, 
        cx: *mut JSContext
    ) -> Result<Option<Rc<OnDebuggerStatement>>> {
        getter!(cx, self, "onDebuggerStatement")
    }

    pub fn set_on_debugger_statement(
        &self,
        cx: *mut JSContext,
        on_debugger_statement: Option<Rc<OnDebuggerStatement>>
    ) -> Result<()> {
        setter!(cx, self, "onDebuggerStatement", on_debugger_statement)
    }

    pub fn get_on_new_script(
        &self, 
        cx: *mut JSContext
    ) -> Result<Option<Rc<OnNewScript>>> {
        getter!(cx, self, "onNewScript")
    }

    pub fn set_on_new_script(
        &self,
        cx: *mut JSContext,
        on_new_script: Option<Rc<OnNewScript>>
    ) -> Result<()> {
        setter!(cx, self, "onNewScript", on_new_script)
    }
}

derive_rooted!(*mut JSObject, Debugger);
