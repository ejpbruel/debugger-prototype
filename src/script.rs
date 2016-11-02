use call::Call;
use convert::{FromJSValue, ToJSValue};
use exception::Result;
use ext::HandleValueArrayExt;
use frame::Frame;
use js::jsapi::{CallArgs, HandleValueArray, JSAutoCompartment, JSContext, JSObject, Value};
use js::jsval;
use object::Object;
use rooted::Rooted;
use source::Source;
use std::rc::Rc;
use trace::TracedBox;
use utils;
use value::ResumptionValue;

pub trait OnHit {
    fn on_hit(&self, cx: *mut JSContext, frame: &Frame) -> Result<ResumptionValue>;
}

impl Call for OnHit {
    unsafe fn call(&self, cx: *mut JSContext, argc: u32, vp: *mut Value) -> bool {
        let args = CallArgs::from_vp(vp, argc);
        match self.on_hit(cx, &Frame::from_js_value(cx, args.get(0)).unwrap()) {
            Ok(result) => result.to_js_value(cx, args.rval()),
            Err(exception) => exception.into_pending_exception(cx)
        }
    }
}

pub struct Breakpoint(TracedBox<*mut JSObject>);

impl Breakpoint{
    pub fn new(cx: *mut JSContext, breakpoint: *mut JSObject) -> Breakpoint {
        Breakpoint(TracedBox::new(cx, breakpoint))
    }
}

derive_rooted!(*mut JSObject, Breakpoint);

derive_convert!(Breakpoint);

pub struct Script(TracedBox<*mut JSObject>);

impl Script {
    pub fn new(cx: *mut JSContext, script: *mut JSObject) -> Script {
        Script(TracedBox::new(cx, script))
    }

    pub fn get_source(&self, cx: *mut JSContext) -> Result<Source> {
        getter!(cx, self, "source")
    }

    pub fn get_url(&self, cx: *mut JSContext) -> Result<String> {
        getter!(cx, self, "url")
    }

    pub fn get_start_line(&self, cx: *mut JSContext) -> Result<usize> {
        getter!(cx, self, "startLine").map(|start_line: u32| start_line as usize)
    }

    pub fn get_line_count(&self, cx: *mut JSContext) -> Result<usize> {
        getter!(cx, self, "lineCount").map(|line_count: u32| line_count as usize)
    }

    pub fn get_global(&self, cx: *mut JSContext) -> Result<Object> {
        getter!(cx, self, "global")
    }

    pub fn get_offsets_for_line(&self, cx: *mut JSContext, line: usize) -> Result<Vec<usize>> {
        method!(cx, self, "getLineOffsets", line as u32).map(|offsets: Vec<u32>| {
            offsets.iter().map(|offset| *offset as usize).collect()
        })
    }

    pub fn add_breakpoint(
        self,
        cx: *mut JSContext,
        offset: usize, on_hit: Rc<OnHit>
    ) -> Result<Breakpoint> {
        unsafe {
            let _ac = JSAutoCompartment::new(cx, self.get());
            rooted!(in (cx) let mut offset_arg = jsval::UndefinedValue());
            try_jsapi!(cx, (offset as u32).to_js_value(cx, offset_arg.handle_mut()));
            rooted!(in (cx) let mut breakpoint_arg = jsval::UndefinedValue());
            try_jsapi!(cx, on_hit.to_js_value(cx, breakpoint_arg.handle_mut()));
            try!(utils::call_method(
                cx,
                self.handle(),
                "setBreakpoint",
                &HandleValueArray::from_slice(&[offset_arg.get(), breakpoint_arg.get()])
            ));
            Ok(Breakpoint::new(cx, breakpoint_arg.to_object()))
        }
    }

    pub fn remove_breakpoint(
        self,
        cx: *mut JSContext,
        offset: usize,
        breakpoint: Breakpoint
    ) -> Result<()> {
        method!(cx, self, "clearBreakpoints", breakpoint, offset as u32)
    }

    pub fn clear_breakpoints(self, cx: *mut JSContext, offset: usize) -> Result<()> {
        method!(cx, self, "clearAllBreakpoints", offset as u32)
    }
}

derive_rooted!(*mut JSObject, Script);

derive_convert!(Script);
