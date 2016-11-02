use call::Call;
use convert::{FromJSValue, NullOr, ToJSValue};
use environment::Environment;
use exception::Result;
use js::jsapi;
use js::jsapi::{CallArgs, HandleValue, JSAutoCompartment, JSContext, JSObject};
use object::Object;
use rooted::Rooted;
use script::Script;
use std::rc::Rc;
use trace::TracedBox;
use utils;
use value::{CompletionValue, ResumptionValue, Value};

pub trait OnPop {
    fn on_pop(
        &self,
        cx: *mut JSContext,
        frame: &Frame,
        value: &CompletionValue
    ) -> Result<ResumptionValue>;
}

impl Call for OnPop {
    unsafe fn call(&self, cx: *mut JSContext, argc: u32, vp: *mut jsapi::Value) -> bool {
        let args = CallArgs::from_vp(vp, argc);
        match self.on_pop(
            cx,
            &Frame::from_js_value(cx, args.get(0)).unwrap(),
            &CompletionValue::from_js_value(cx, args.get(1)).unwrap()
        ) {
            Ok(result) => result.to_js_value(cx, args.rval()),
            Err(exception) => exception.into_pending_exception(cx)
        }
    }
}

pub trait OnStep {
    fn on_step(&self, cx: *mut JSContext, frame: &Frame) -> Result<ResumptionValue>;
}

impl Call for OnStep {
    unsafe fn call(&self, cx: *mut JSContext, argc: u32, vp: *mut jsapi::Value) -> bool {
        let args = CallArgs::from_vp(vp, argc);
        match self.on_step(cx, &Frame::from_js_value(cx, args.get(0)).unwrap()) {
            Ok(result) => result.to_js_value(cx, args.rval()),
            Err(exception) => exception.into_pending_exception(cx)
        }
    }
}

pub enum FrameType {
    Call,
    Eval,
    Global,
    Module
}

impl FromJSValue for FrameType {
    unsafe fn from_js_value(cx: *mut JSContext, v: HandleValue) -> Result<Self> {
        String::from_js_value(cx, v).map(|string| {
            if string == "call" {
                FrameType::Call
            } else if string == "eval" {
                FrameType::Eval
            } else if string == "global" {
                FrameType::Global
            } else if string == "module" {
                FrameType::Module
            } else {
                panic!();
            }
        })
    }
}

pub enum FrameImplementation {
    Interpreter,
    Baseline,
    Ion
}

impl FromJSValue for FrameImplementation {
    unsafe fn from_js_value(cx: *mut JSContext, v: HandleValue) -> Result<Self> {
        String::from_js_value(cx, v).map(|string| {
            if string == "interpreter" {
                FrameImplementation::Interpreter
            } else if string == "baseline" {
                FrameImplementation::Baseline
            } else if string == "ion" {
                FrameImplementation::Ion
            } else {
                panic!();
            }
        })
    }
}

pub struct Arguments(TracedBox<*mut JSObject>);

impl Arguments {
    pub fn new(cx: *mut JSContext, arguments: *mut JSObject) -> Arguments {
        Arguments(TracedBox::new(cx, arguments))
    }

    pub fn get_length(&self, cx: *mut JSContext) -> Result<usize> {
        getter!(cx, self, "length").map(|length: u32| length as usize)
    }

    pub fn get_element(&self, cx: *mut JSContext, index: usize) -> Result<Value> {
        unsafe {
            let _ac = JSAutoCompartment::new(cx, self.get());
            utils::get_element(cx, self.handle(), index)
        }
    }

    pub fn set_element(&self, cx: *mut JSContext, index: usize, value: &Value) -> Result<()> {
        unsafe {
            let _ac = JSAutoCompartment::new(cx, self.get());
            try_jsapi!(cx, utils::set_element(cx, self.handle(), index, value));
            Ok(())
        }
    }
}

derive_rooted!(*mut JSObject, Arguments);

derive_convert!(Arguments);

pub struct Frame(TracedBox<*mut JSObject>);

impl Frame {
    pub fn new(cx: *mut JSContext, frame: *mut JSObject) -> Frame {
        Frame(TracedBox::new(cx, frame))
    }

    pub fn get_is_live(&self, cx: *mut JSContext) -> Result<bool> {
        getter!(cx, self, "live")
    }

    pub fn get_is_constructing(&self, cx: *mut JSContext) -> Result<bool> {
        getter!(cx, self, "constructing")
    }

    pub fn get_type(&self, cx: *mut JSContext) -> Result<FrameType> {
        getter!(cx, self, "type")
    }

    pub fn get_implementation(&self, cx: *mut JSContext) -> Result<FrameImplementation> {
        getter!(cx, self, "implementation")
    }

    pub fn get_depth(&self, cx: *mut JSContext) -> Result<usize> {
        getter!(cx, self, "depth").map(|depth: u32| depth as usize)
    }

    pub fn get_older(&self, cx: *mut JSContext) -> Result<Option<Frame>> {
        getter!(cx, self, "older").map(|older| NullOr::<Frame>::into_option(older))
    }

    pub fn get_callee(&self, cx: *mut JSContext) -> Result<Option<Object>> {
        getter!(cx, self, "callee").map(|callee| NullOr::<Object>::into_option(callee))
    }

    pub fn get_this(&self, cx: *mut JSContext) -> Result<Value> {
        getter!(cx, self, "this")
    }

    pub fn get_arguments(&self, cx: *mut JSContext) -> Result<Option<Arguments>> {
        getter!(cx, self, "arguments").map(|arguments| {
            NullOr::<Arguments>::into_option(arguments)
        })
    }

    pub fn get_script(&self, cx: *mut JSContext) -> Result<Option<Script>> {
        getter!(cx, self, "script").map(|script| NullOr::<Script>::into_option(script))
    }

    pub fn get_offset(&self, cx: *mut JSContext) -> Result<usize> {
        getter!(cx, self, "offset").map(|offset: u32| offset as usize)
    }

    pub fn get_environment(&self, cx: *mut JSContext) -> Result<Option<Environment>> {
        getter!(cx, self, "environment").map(|environment| {
            NullOr::<Environment>::into_option(environment)
        })
    }

    pub fn get_on_pop(&self, cx: *mut JSContext) -> Result<Option<Rc<OnPop>>> {
        getter!(cx, self, "onPop")
    }

    pub fn set_on_pop(&self, cx: *mut JSContext, on_pop: Option<Rc<OnPop>>) -> Result<()> {
        setter!(cx, self, "onPop", on_pop)
    }

    pub fn get_on_step(&self, cx: *mut JSContext) -> Result<Option<Rc<OnStep>>> {
        getter!(cx, self, "onStep")
    }

    pub fn set_on_step(&self, cx: *mut JSContext, on_step: Option<Rc<OnStep>>) -> Result<()> {
        setter!(cx, self, "onStep", on_step)
    }
}

derive_rooted!(*mut JSObject, Frame);

derive_convert!(Frame);
