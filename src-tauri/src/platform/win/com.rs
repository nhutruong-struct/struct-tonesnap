//! COM runs on short-lived dedicated MTA threads so its state never leaks into Tauri's threads.

use std::sync::{mpsc, Once};
use std::time::Duration;

use windows::Win32::System::Com::{
    CoIncrementMTAUsage, CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED,
};

/// Runs `f` on a fresh thread initialized for COM (MTA) and waits for its result. `None` when
/// the thread panicked.
pub fn on_mta_thread<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> Option<T> {
    start_on_mta_thread(f).recv().ok()
}

/// Like `on_mta_thread`, but waits at most `limit`. A call into another process (Word) blocks
/// for as long as that process does not answer; past the limit the thread is left to finish on
/// its own and its result is dropped. `None` on timeout or panic.
pub fn on_mta_thread_within<T: Send + 'static>(
    limit: Duration,
    f: impl FnOnce() -> T + Send + 'static,
) -> Option<T> {
    start_on_mta_thread(f).recv_timeout(limit).ok()
}

/// Starts `f` on a fresh thread initialized for COM (MTA). Its result arrives on the returned
/// channel, which closes without one if the thread panicked.
pub fn start_on_mta_thread<T: Send + 'static>(
    f: impl FnOnce() -> T + Send + 'static,
) -> mpsc::Receiver<T> {
    keep_mta_alive();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        // SAFETY: a fresh thread with no COM state yet; the reserved argument is None.
        let initialized = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.is_ok();
        let _ = tx.send(f());
        if initialized {
            // SAFETY: balances the successful CoInitializeEx above, on the same thread, after
            // `f` (and every COM object it created) has finished.
            unsafe { CoUninitialize() };
        }
    });
    rx
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

#[cfg(test)]
mod tests {
    use std::thread::sleep;

    use super::*;

    #[test]
    fn a_quick_result_comes_back_and_a_slow_one_is_dropped() {
        assert_eq!(on_mta_thread_within(Duration::from_secs(5), || 7), Some(7));
        let slow = on_mta_thread_within(Duration::from_millis(20), || {
            sleep(Duration::from_millis(500));
            7
        });
        assert_eq!(slow, None);
    }
}
