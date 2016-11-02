use exception::Result;
use js::jsapi::{JSContext, JSObject};
use rooted::Rooted;
use trace::TracedBox;

pub struct Source(TracedBox<*mut JSObject>);

impl Source {
    pub fn new(cx: *mut JSContext, source: *mut JSObject) -> Source {
        Source(TracedBox::new(cx, source))
    }

    pub fn get_text(&self, cx: *mut JSContext) -> Result<String> {
        getter!(cx, self, "text")
    }
}

derive_rooted!(*mut JSObject, Source);

derive_convert!(Source);
