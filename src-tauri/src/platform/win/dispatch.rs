//! Late-bound COM automation (`IDispatch`): objects called by member name with `VARIANT`
//! arguments, so an app's object model (Word's) is used without its type library.

use std::mem::ManuallyDrop;
use std::thread::sleep;
use std::time::Duration;

use windows::core::{BSTR, GUID, PCWSTR};
use windows::Win32::Foundation::{E_UNEXPECTED, RPC_E_CALL_REJECTED, RPC_E_SERVERCALL_RETRYLATER};
use windows::Win32::System::Com::{
    IDispatch, DISPATCH_FLAGS, DISPATCH_METHOD, DISPATCH_PROPERTYGET, DISPATCH_PROPERTYPUT,
    DISPPARAMS,
};
use windows::Win32::System::Ole::DISPID_PROPERTYPUT;
use windows::Win32::System::Variant::{
    VariantClear, VARIANT, VT_BOOL, VT_BSTR, VT_DISPATCH, VT_I4,
};

/// A VARIANT that is cleared when dropped.
#[repr(transparent)]
pub struct Value(VARIANT);

impl Value {
    pub fn int(n: i32) -> Self {
        let mut v = VARIANT::default();
        // SAFETY: a zeroed VARIANT is VT_EMPTY; the tag and the member it names are set together.
        unsafe {
            let inner = &mut *v.Anonymous.Anonymous;
            inner.vt = VT_I4;
            inner.Anonymous.lVal = n;
        }
        Self(v)
    }

    pub fn text(s: &str) -> Self {
        let mut v = VARIANT::default();
        // SAFETY: as in `int`; the BSTR is owned by the VARIANT and freed by `VariantClear`.
        unsafe {
            let inner = &mut *v.Anonymous.Anonymous;
            inner.vt = VT_BSTR;
            inner.Anonymous.bstrVal = ManuallyDrop::new(BSTR::from(s));
        }
        Self(v)
    }

    #[cfg(test)]
    pub fn object(object: &Object) -> Self {
        let mut v = VARIANT::default();
        // SAFETY: as in `int`; the VARIANT holds its own reference, released by `VariantClear`.
        unsafe {
            let inner = &mut *v.Anonymous.Anonymous;
            inner.vt = VT_DISPATCH;
            inner.Anonymous.pdispVal = ManuallyDrop::new(Some(object.0.clone()));
        }
        Self(v)
    }

    #[cfg(test)]
    pub fn flag(on: bool) -> Self {
        let mut v = VARIANT::default();
        // SAFETY: as in `int`.
        unsafe {
            let inner = &mut *v.Anonymous.Anonymous;
            inner.vt = VT_BOOL;
            inner.Anonymous.boolVal =
                windows::Win32::Foundation::VARIANT_BOOL(if on { -1 } else { 0 });
        }
        Self(v)
    }

    pub fn as_bool(&self) -> Option<bool> {
        // SAFETY: as in `as_int`.
        unsafe {
            let inner = &self.0.Anonymous.Anonymous;
            (inner.vt == VT_BOOL).then(|| inner.Anonymous.boolVal.0 != 0)
        }
    }

    pub fn as_int(&self) -> Option<i32> {
        // SAFETY: the member is read only when the tag says it holds it.
        unsafe {
            let inner = &self.0.Anonymous.Anonymous;
            (inner.vt == VT_I4).then(|| inner.Anonymous.lVal)
        }
    }

    pub fn as_text(&self) -> Option<String> {
        // SAFETY: as in `as_int`.
        unsafe {
            let inner = &self.0.Anonymous.Anonymous;
            (inner.vt == VT_BSTR).then(|| inner.Anonymous.bstrVal.to_string())
        }
    }

    pub fn as_object(&self) -> Option<Object> {
        // SAFETY: as in `as_int`; the interface is cloned (AddRef), the VARIANT keeps its own.
        unsafe {
            let inner = &self.0.Anonymous.Anonymous;
            if inner.vt != VT_DISPATCH {
                return None;
            }
            (*inner.Anonymous.pdispVal).clone().map(Object)
        }
    }
}

impl Drop for Value {
    fn drop(&mut self) {
        // SAFETY: the VARIANT is valid (built above or written by `Invoke`) and cleared once.
        unsafe {
            let _ = VariantClear(&mut self.0);
        }
    }
}

pub fn unexpected() -> windows::core::Error {
    E_UNEXPECTED.into()
}

/// A Word object, called by name.
pub struct Object(pub IDispatch);

impl Object {
    fn invoke(
        &self,
        name: &str,
        flags: DISPATCH_FLAGS,
        mut args: Vec<Value>,
    ) -> windows::core::Result<Value> {
        let wide: Vec<u16> = name.encode_utf16().chain([0]).collect();
        let mut id = 0;
        // SAFETY: `wide` is a NUL-terminated name that outlives the call; one name, one id.
        unsafe {
            self.0
                .GetIDsOfNames(&GUID::zeroed(), &PCWSTR(wide.as_ptr()), 1, 0, &mut id)?
        };
        // DISPPARAMS lists the arguments last first; a property put names its value.
        args.reverse();
        let put = flags == DISPATCH_PROPERTYPUT;
        let mut named = DISPID_PROPERTYPUT;
        let params = DISPPARAMS {
            rgvarg: args.as_mut_ptr().cast(),
            rgdispidNamedArgs: if put {
                &mut named
            } else {
                std::ptr::null_mut()
            },
            cArgs: args.len() as u32,
            cNamedArgs: u32::from(put),
        };
        let mut result = Value(VARIANT::default());
        // SAFETY: `params` points into `args` and `named`, which outlive the call (`Value` is a
        // transparent VARIANT); the result is cleared when `result` drops.
        unsafe {
            self.0.Invoke(
                id,
                &GUID::zeroed(),
                0,
                flags,
                &params,
                Some(&mut result.0),
                None,
                None,
            )?
        };
        Ok(result)
    }

    pub fn object(&self, name: &str) -> windows::core::Result<Object> {
        self.invoke(name, DISPATCH_PROPERTYGET, Vec::new())?
            .as_object()
            .ok_or_else(unexpected)
    }

    pub fn int(&self, name: &str) -> windows::core::Result<i32> {
        self.invoke(name, DISPATCH_PROPERTYGET, Vec::new())?
            .as_int()
            .ok_or_else(unexpected)
    }

    pub fn flag(&self, name: &str) -> windows::core::Result<bool> {
        self.invoke(name, DISPATCH_PROPERTYGET, Vec::new())?
            .as_bool()
            .ok_or_else(unexpected)
    }

    pub fn text(&self, name: &str) -> windows::core::Result<String> {
        self.invoke(name, DISPATCH_PROPERTYGET, Vec::new())?
            .as_text()
            .ok_or_else(unexpected)
    }

    pub fn call(&self, name: &str, args: Vec<Value>) -> windows::core::Result<Value> {
        self.invoke(name, DISPATCH_METHOD, args)
    }

    pub fn put(&self, name: &str, value: Value) -> windows::core::Result<()> {
        self.invoke(name, DISPATCH_PROPERTYPUT, vec![value])
            .map(drop)
    }
}

/// Runs `f` again while the server turns calls away because it is busy (a dialog, a long operation).
pub fn retry<T>(mut f: impl FnMut() -> windows::core::Result<T>) -> windows::core::Result<T> {
    for _ in 0..3 {
        match f() {
            Err(e)
                if e.code() == RPC_E_CALL_REJECTED || e.code() == RPC_E_SERVERCALL_RETRYLATER =>
            {
                sleep(Duration::from_millis(50))
            }
            other => return other,
        }
    }
    f()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_busy_calls_are_tried_again() {
        let mut calls = 0;
        let busy = retry(|| -> windows::core::Result<()> {
            calls += 1;
            Err(RPC_E_CALL_REJECTED.into())
        });
        assert!(busy.is_err());
        assert_eq!(calls, 4);

        let mut calls = 0;
        let other = retry(|| -> windows::core::Result<()> {
            calls += 1;
            Err(unexpected())
        });
        assert!(other.is_err());
        assert_eq!(calls, 1);

        let mut calls = 0;
        let later = retry(|| {
            calls += 1;
            if calls < 3 {
                Err(RPC_E_SERVERCALL_RETRYLATER.into())
            } else {
                Ok(calls)
            }
        });
        assert_eq!(later.ok(), Some(3));
    }
}
