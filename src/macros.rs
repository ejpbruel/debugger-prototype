macro_rules! c_str {
    ($str:expr) => {
        concat!($str, "\0").as_ptr() as *const ::std::os::raw::c_char
    }
}

macro_rules! derive_rooted {
    ($T:ty, $U:ty) => {
        impl ::rooted::Rooted<$T> for $U {
            fn get(&self) -> $T {
                self.0.get()
            }

            fn handle(&self) -> ::js::jsapi::Handle<$T> {
                self.0.handle()
            }

            fn handle_mut(&mut self) -> ::js::jsapi::MutableHandle<$T> {
                self.0.handle_mut()
            }
        }
    }
}

macro_rules! derive_convert {
    ($T:ty) => {
        impl ::convert::FromJSValue for $T {
            unsafe fn from_js_value(
                cx: *mut ::js::jsapi::JSContext,
                v: ::js::jsapi::HandleValue
            ) -> ::exception::Result<Self> {
                ::convert::FromJSValue::from_js_value(cx, v).map(|obj| <$T>::new(cx, obj))
            }
        }

        impl ::convert::ToJSValue for $T {
            unsafe fn to_js_value(
                &self,
                cx: *mut ::js::jsapi::JSContext,
                rval: ::js::jsapi::MutableHandleValue
            ) -> bool {
                self.get().to_js_value(cx, rval)
            }
        }
    }
}


macro_rules! try_jsapi {
    ($expr:expr) => {{
        let result = $expr;
        if result as usize == 0 {
            return false;
        }
        result
    }};
    ($cx:expr, $expr:expr) => {{
        let result = $expr;
        if result as usize == 0 {
            return Err(::exception::Exception::from_pending_exception($cx));
        }
        result
    }}
}

macro_rules! getter {
    ($cx:expr, $obj:expr, $name:expr) => {
        unsafe {
            let _ac = ::js::jsapi::JSAutoCompartment::new($cx, $obj.get());
            ::utils::get_property($cx, $obj.handle(), $name)
        }
    }
}

macro_rules! setter {
    ($cx:expr, $obj:expr, $name:expr, $value:expr) => {
        unsafe {
            let _ac = ::js::jsapi::JSAutoCompartment::new($cx, $obj.get());
            try_jsapi!($cx, ::utils::set_property($cx, $obj.handle(), $name, $value));
            Ok(())
        }
    }
}

macro_rules! method {
    ($cx:expr, $obj:expr, $name:expr) => {
        unsafe {
            let _ac = ::js::jsapi::JSAutoCompartment::new($cx, $obj.get());
            ::utils::call_method(
                $cx,
                $obj.handle(),
                $name,
                &::js::jsapi::HandleValueArray::new()
            )
        }
    };
    ($cx:expr, $obj:expr, $name:expr, $($arg:expr),*) => {
        unsafe {
            let _ac = ::js::jsapi::JSAutoCompartment::new($cx, $obj.get());
            let mut args = Vec::new();
            $(
                rooted!(in ($cx) let mut arg = ::js::jsval::UndefinedValue());
                try_jsapi!($cx, $arg.to_js_value($cx, arg.handle_mut()));
                args.push(arg.get());
            )*
            ::utils::call_method(
                $cx,
                $obj.handle(),
                $name,
                &::js::jsapi::HandleValueArray::from_slice(&args)
            )
        }
    }
}
