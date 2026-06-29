; NSIS installer hooks for Agent Switch Tools.
;
; The app was renamed from "Claude Tools" to "Agent Switch Tools". Because the
; product name changed, the new installer does not recognise the old install and
; would otherwise leave it behind. This hook silently uninstalls the legacy
; "Claude Tools" build before installing the new one.

Var OLD_UNINSTALL_STRING

!macro NSIS_HOOK_PREINSTALL
  ; Per-machine install (HKLM)
  ReadRegStr $OLD_UNINSTALL_STRING HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Claude Tools" "QuietUninstallString"
  ${If} $OLD_UNINSTALL_STRING != ""
    DetailPrint "Removing legacy Claude Tools (per-machine)..."
    ExecWait '$OLD_UNINSTALL_STRING /S'
  ${EndIf}

  ; Per-user install (HKCU)
  ReadRegStr $OLD_UNINSTALL_STRING HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Claude Tools" "QuietUninstallString"
  ${If} $OLD_UNINSTALL_STRING != ""
    DetailPrint "Removing legacy Claude Tools (per-user)..."
    ExecWait '$OLD_UNINSTALL_STRING /S'
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTINSTALL
!macroend

!macro NSIS_HOOK_PREUNINSTALL
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
!macroend
