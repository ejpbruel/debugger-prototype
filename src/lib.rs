#[macro_use]
extern crate js;

#[macro_use]
mod macros;

mod call;
mod convert;
mod ext;
mod rooted;
mod trace;
mod utils;

pub mod debugger;
pub mod environment;
pub mod exception;
pub mod frame;
pub mod object;
pub mod script;
pub mod source;
pub mod value;

pub use debugger::{Debugger, OnDebuggerStatement, OnNewScript};
pub use exception::{Exception, Result};
pub use environment::Environment;
pub use frame::{Arguments, Frame};
pub use object::{Object, PropertyDescriptor};
pub use source::Source;
pub use script::Script;
pub use value::{CompletionValue, ResumptionValue, Value};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        use debugger::{Debugger, OnDebuggerStatement, OnNewScript};
        use exception::Result;
        use frame::Frame;
        use js::jsapi::JSContext;
        use js::jsval::UndefinedValue;
        use js::rust::Runtime;
        use script::Script;
        use std::ptr;
        use std::rc::Rc;
        use utils;
        use value::{ResumptionValue, Value};

        struct Handler;

        impl OnNewScript for Handler {
            fn on_new_script(&self, cx: *mut JSContext, script: &Script) -> Result<()> {
                let source = try!(script.get_source(cx));
                let text = try!(source.get_text(cx));
                println!("{}", text);
                Ok(())
            }
        }

        impl OnDebuggerStatement for Handler {
            fn on_debugger_statement(
                &self,
                cx: *mut JSContext,
                frame: &Frame
            ) -> Result<ResumptionValue> {
                let arguments = try!(frame.get_arguments(cx)).unwrap();
                let argument = try!(arguments.get_element(cx, 0));
                match argument {
                    Value::String(s) => {
                        println!("{}", s);
                    }
                    _ => {
                        panic!()
                    }
                }
                Ok(None)
            }
        }

        let runtime = Runtime::new();
        rooted!(in (runtime.cx()) let mut global = ptr::null_mut());
        unsafe {
            assert!(utils::new_global_object(runtime.cx(), global.handle_mut()));
        }
        let debugger = Debugger::new(runtime.cx());
        debugger.add_debuggee(runtime.cx(), global.handle()).unwrap();
        let handler = Rc::new(Handler);
        debugger.set_on_new_script(runtime.cx(), Some(handler.clone())).unwrap();
        debugger.set_on_debugger_statement(runtime.cx(), Some(handler.clone())).unwrap();
        rooted!(in (runtime.cx()) let mut rval = UndefinedValue());
        runtime.evaluate_script(global.handle(), r#"
            function f(x) {
                debugger;
            }

            f("TEST");
        "#, "test", 0, rval.handle_mut()).unwrap();
    }
}
