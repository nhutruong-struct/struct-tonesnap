; ToneSnap's additions to Tauri's NSIS installer (`bundle.windows.nsis.installerHooks`).
; Tauri's uninstaller already removes the "Start with Windows" `Run` entry and, when the user
; ticks "Delete the application data", the settings and WebView folders.

; CRED_TYPE_GENERIC is 1. A credential that does not exist just makes the call fail, which is fine.
!macro TONESNAP_DELETE_CREDENTIAL TARGET
  System::Call 'advapi32::CredDeleteW(w "${TARGET}", i 1, i 0) i'
!macroend

; The API keys (secrets.rs) are application data too: they go with the same checkbox, so a plain
; uninstall and reinstall keeps them. The keyring crate names each credential
; "<account>.<service>"; a test in secrets.rs keeps these names in step. Only the names are used,
; the keys are never read. An update (/UPDATE) keeps everything.
!macro NSIS_HOOK_POSTUNINSTALL
  ${If} $DeleteAppDataCheckboxState = 1
  ${AndIf} $UpdateMode <> 1
    !insertmacro TONESNAP_DELETE_CREDENTIAL "anthropic_api_key.ToneSnap"
    !insertmacro TONESNAP_DELETE_CREDENTIAL "gemini_api_key.ToneSnap"
    !insertmacro TONESNAP_DELETE_CREDENTIAL "groq_api_key.ToneSnap"
    !insertmacro TONESNAP_DELETE_CREDENTIAL "openrouter_api_key.ToneSnap"
  ${EndIf}
!macroend
