#[cfg(target_os = "android")]
mod android_entry {
    use std::ffi::c_void;

    #[unsafe(no_mangle)]
    pub extern "C" fn ANativeActivity_onCreate(
        activity: *mut c_void,
        _vm: *mut c_void,
        _saved_state: *mut c_void,
    ) {
        let _ = activity;
    }
}

#[cfg(target_os = "android")]
pub use android_entry::ANativeActivity_onCreate;

#[cfg(not(target_os = "android"))]
pub fn placeholder() {}
