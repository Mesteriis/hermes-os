use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::path::Path;

use libloading::Library;
use serde_json::Value;

use crate::integrations::telegram::client::errors::TelegramError;

use super::library_paths::tdjson_library_candidates;

type TdJsonClientCreate = unsafe extern "C" fn() -> *mut c_void;
type TdJsonClientSend = unsafe extern "C" fn(*mut c_void, *const c_char);
type TdJsonClientReceive = unsafe extern "C" fn(*mut c_void, f64) -> *const c_char;
type TdJsonClientExecute = unsafe extern "C" fn(*mut c_void, *const c_char) -> *const c_char;
type TdJsonClientDestroy = unsafe extern "C" fn(*mut c_void);

pub(crate) fn runtime_available(configured_path: Option<&Path>) -> bool {
    TdJsonLibrary::load(configured_path).is_ok()
}

pub(crate) struct TdJsonLibrary {
    create: TdJsonClientCreate,
    send: TdJsonClientSend,
    receive: TdJsonClientReceive,
    execute: TdJsonClientExecute,
    destroy: TdJsonClientDestroy,
    _library: Library,
}

impl TdJsonLibrary {
    pub(crate) fn load(configured_path: Option<&Path>) -> Result<Self, TelegramError> {
        let candidates = tdjson_library_candidates(configured_path);
        let mut load_errors = Vec::new();

        for candidate in candidates {
            let library = {
                // SAFETY: Loading a dynamic library is unsafe because the symbols may not
                // match the expected ABI. Symbols are verified immediately below and kept
                // alive by storing the Library inside TdJsonLibrary.
                unsafe { Library::new(&candidate) }
            };
            match library {
                Ok(library) => return Self::from_library(library, &candidate),
                Err(error) => {
                    load_errors.push(format!("{}: {error}", candidate.display()));
                    if configured_path.is_some() {
                        break;
                    }
                }
            }
        }

        Err(TelegramError::TdlibRuntimeUnavailable(format!(
            "unable to load libtdjson; tried {}",
            load_errors.join("; ")
        )))
    }

    fn from_library(library: Library, candidate: &Path) -> Result<Self, TelegramError> {
        let create = load_symbol(&library, b"td_json_client_create\0", candidate)?;
        let send = load_symbol(&library, b"td_json_client_send\0", candidate)?;
        let receive = load_symbol(&library, b"td_json_client_receive\0", candidate)?;
        let execute = load_symbol(&library, b"td_json_client_execute\0", candidate)?;
        let destroy = load_symbol(&library, b"td_json_client_destroy\0", candidate)?;

        Ok(Self {
            create,
            send,
            receive,
            execute,
            destroy,
            _library: library,
        })
    }

    pub(crate) fn create_client(self) -> Result<TdJsonClient, TelegramError> {
        let client = {
            // SAFETY: The function pointer was loaded from libtdjson with the documented
            // C ABI and returns an opaque TDLib client pointer owned by the caller.
            unsafe { (self.create)() }
        };
        if client.is_null() {
            return Err(TelegramError::TdlibRuntime(
                "td_json_client_create returned null".to_owned(),
            ));
        }

        Ok(TdJsonClient {
            client,
            library: self,
        })
    }
}

pub(crate) struct TdJsonClient {
    client: *mut c_void,
    library: TdJsonLibrary,
}

impl TdJsonClient {
    pub(crate) fn send_json(&self, request: &Value) -> Result<(), TelegramError> {
        let request = CString::new(request.to_string()).map_err(|_| {
            TelegramError::TdlibRuntime("TDLib request contained an interior NUL byte".to_owned())
        })?;
        // SAFETY: The client pointer is created by td_json_client_create and remains
        // valid until Drop calls td_json_client_destroy. CString is NUL-terminated
        // and lives for the duration of the call.
        unsafe {
            (self.library.send)(self.client, request.as_ptr());
        }
        Ok(())
    }

    pub(crate) fn receive_json(
        &self,
        timeout_seconds: f64,
    ) -> Result<Option<Value>, TelegramError> {
        let response = {
            // SAFETY: TDLib owns the returned pointer until the next receive/execute
            // call on this client. The string is copied into an owned Rust String
            // before another TDLib call can invalidate it.
            unsafe { (self.library.receive)(self.client, timeout_seconds) }
        };
        if response.is_null() {
            return Ok(None);
        }

        let response = {
            // SAFETY: td_json_client_receive returns a NUL-terminated UTF-8 JSON string
            // pointer or NULL. NULL was handled above.
            unsafe { CStr::from_ptr(response) }
        }
        .to_str()
        .map_err(|error| TelegramError::TdlibRuntime(format!("invalid TDLib JSON UTF-8: {error}")))?
        .to_owned();

        serde_json::from_str(&response)
            .map(Some)
            .map_err(|error| TelegramError::TdlibRuntime(format!("invalid TDLib JSON: {error}")))
    }

    pub(crate) fn execute_json(&self, request: &Value) -> Result<Option<Value>, TelegramError> {
        let request = CString::new(request.to_string()).map_err(|_| {
            TelegramError::TdlibRuntime("TDLib request contained an interior NUL byte".to_owned())
        })?;
        let response = {
            // SAFETY: The client pointer and request CString satisfy td_json_client_execute
            // requirements. The returned pointer is copied before the next TDLib call.
            unsafe { (self.library.execute)(self.client, request.as_ptr()) }
        };
        if response.is_null() {
            return Ok(None);
        }

        let response = {
            // SAFETY: td_json_client_execute returns a NUL-terminated JSON string pointer
            // or NULL. NULL was handled above.
            unsafe { CStr::from_ptr(response) }
        }
        .to_str()
        .map_err(|error| TelegramError::TdlibRuntime(format!("invalid TDLib JSON UTF-8: {error}")))?
        .to_owned();

        serde_json::from_str(&response)
            .map(Some)
            .map_err(|error| TelegramError::TdlibRuntime(format!("invalid TDLib JSON: {error}")))
    }
}

impl Drop for TdJsonClient {
    fn drop(&mut self) {
        if !self.client.is_null() {
            // SAFETY: The pointer was created by td_json_client_create and is destroyed
            // exactly once here, before the backing library is unloaded.
            unsafe {
                (self.library.destroy)(self.client);
            }
            self.client = std::ptr::null_mut();
        }
    }
}

fn load_symbol<T: Copy>(
    library: &Library,
    name: &'static [u8],
    candidate: &Path,
) -> Result<T, TelegramError> {
    let symbol = {
        // SAFETY: Symbol type T is the exact C ABI function pointer expected for the
        // named TDLib JSON symbol. The Library is retained for at least as long as
        // copied function pointers can be called.
        unsafe { library.get::<T>(name) }
    }
    .map_err(|error| {
        let symbol_name = name.strip_suffix(b"\0").unwrap_or(name);
        TelegramError::TdlibRuntimeUnavailable(format!(
            "libtdjson `{}` is missing symbol `{}`: {error}",
            candidate.display(),
            String::from_utf8_lossy(symbol_name)
        ))
    })?;
    Ok(*symbol)
}
