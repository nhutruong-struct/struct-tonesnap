/** Whether the interface runs on macOS: its shortcuts, keychain and wording differ from Windows. */
export function isMac(): boolean {
  return /Mac/i.test(navigator.userAgent);
}
