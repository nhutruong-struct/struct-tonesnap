//! COM runs on short-lived dedicated MTA threads so its state never leaks into Tauri's threads.

use std::sync::Once;

use windows::Win32::System::Com::{
    CoIncrementMTAUsage, CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED,
};

/// Runs `f` on a fresh thread initialized for COM (MTA) and waits for its result. `None` when
/// the thread panicked.
pub fn on_mta_thread<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> Option<T> {
    keep_mta_alive();
    std::thread::spawn(move || {
        // SAFETY: a fresh thread with no COM state yet; the reserved argument is None.
        let initialized = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.is_ok();
        let result = f();
        if initialized {
            // SAFETY: balances the successful CoInitializeEx above, on the same thread, after
            // `f` (and every COM object it created) has finished.
            unsafe { CoUninitialize() };
        }
        result
    })
    .join()
    .ok()
}

/// Keeps the process's MTA alive between calls. Otherwise the last thread leaving it tears it
/// down and unloads in-process servers, while the `windows` crate keeps WinRT factories (such
/// as OcrEngine's) cached: the next call on a new thread would use a dangling factory and crash.
///
/// Side effect: afterwards, threads that never initialize COM run in the implicit MTA, so a COM
/// call made outside `on_mta_thread` could appear to work. Keep every COM call in here anyway.
fn keep_mta_alive() {
    static KEEP: Once = Once::new();
    // The usage cookie is never released: the MTA should live as long as the process.
    // SAFETY: CoIncrementMTAUsage takes no input and may be called from any thread.
    KEEP.call_once(|| unsafe {
        let _ = CoIncrementMTAUsage();
    });
}
