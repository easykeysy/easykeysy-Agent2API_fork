; Agent2API's NSIS installer hooks (wired in via tauri.conf.json's
; bundle.windows.nsis.installerHooks 引入）。
;
; Execution point: the template runs this file only after all !includes (MUI2 / utils.nsh / official plugin dirs)
; have been processed (nsis-template.nsi line 54: `!include "{{installer_hooks}}"`),
; and the hook macros (NSIS_HOOK_PREINSTALL etc.) are expanded inline inside Sections. So every
; fallback macro below sees the official definitions at expansion time — if the official one exists we don't inject; only when it's missing do we
; fill in a minimal safe implementation. In either case makensis no longer fails with
; "!insertmacro: macro named ... not found!".
;
; ── What's officially missing, and what we patch in ─────────────────────────────────────────────
; · RestartManager_StartSession / RegisterFile / EndSession
;   (in the tauri-bundler v2.11.4 release, utils.nsh references it without defining it) -> patched into
;   a minimal empty-registration implementation: no files registered -> RmGetList always returns 0 ->
;   the ERROR_MORE_DATA branch never fires -> the check always takes the "app not running" path.
;   When the official implementation is present this whole block is skipped; behavior is identical to the official one.
; · RmRebootReasonPermissionDenied / RmForceShutdown (RmErrorCodes.nsh
;   is not shipped with the release) -> values filled in from the Microsoft Restart Manager headers.
; · Official CheckIfAppIsRunning absent (older utils.nsh scenario) -> a tasklist-based
;   probe version (no kill; GUI prompt; canceling aborts the install).
;
; ── What the hook bodies themselves patch in ────────────────────────────────────────────
; 主程序的产物名从 `workbuddy-proxy-desktop.exe` 改成了 `agent2api.exe`
; （tauri.conf.json 的 mainBinaryName）。模板自带的「应用正在运行」检查只认
; 当前这一个名字（installer.nsi 里的 `CheckIfAppIsRunning "${MAINBINARYNAME}.exe"`），
; 于是「用户手动运行新安装包、旧版本还开着」时模板认不出那个旧进程：
; 安装本身会成功，但紧接着模板删旧主程序的那一步
; （`Delete "$INSTDIR\$OldMainBinaryName"`）会因为文件被占用而静默失败，
; 旧 exe 就留在安装目录里 —— 白占十几 MB 之外，卸载时结尾那句
; `RMDir "$INSTDIR"` 也会因为这个残留文件删不掉整个目录。
;
; 这里在复制文件之前对旧名进程做一次同样的检查，交互与模板逐字一致：
; 静默安装直接结束它，图形安装弹「是否结束该程序」，用户点取消就中止安装。
;
; ── When this can be deleted ────────────────────────────────────────────────
; 这是过渡期补丁，只在「用户机器上还可能装着改名前的版本」时有意义。
; 改名后的版本发出足够久（用户基本都升过一次）之后，磁盘上不会再出现旧名主程序，
; 届时删掉本文件与 tauri.conf.json 里的 installerHooks 即可。
; 若主程序将来再次改名，按同一思路把「上一次的名字」加进来。
; (Same for the RestartManager / RmErrorCodes fallback blocks: the day tauri-bundler
; ships the official utils.nsh with its release, these !ifmacrodef branches die on their own and can be deleted wholesale.)
!ifmacrodef RestartManager_StartSession
!else
!macro RestartManager_StartSession outreg
  ; Minimal fallback when the official implementation is missing: empty session (no files registered -> RmGetList returns 0 ->
  ; takes the "app not running" path).
  System::Call 'kernel32::CreateMutex(w "", i 0, p 0) i .r0'
  StrCpy "${outreg}" ""
  ${If} $0 = 0
    StrCpy "${outreg}" "a2a-empty"
  ${EndIf}
!macroend
!endif
!ifmacrodef RestartManager_RegisterFile
!else
!macro RestartManager_RegisterFile handle file
!macroend
!endif
!ifmacrodef RestartManager_EndSession
!else
!macro RestartManager_EndSession handle
!macroend
!endif

!define /ifndef RmRebootReasonPermissionDenied 0x00000020
!define /ifndef RmForceShutdown 1

!ifmacrodef CheckIfAppIsRunning
!else
; Fallback when the official utils.nsh is absent: tasklist probe + optional taskkill.
; GUI install -> prompt "end this program?"; confirming runs taskkill in the background then continues,
; canceling aborts the install; silent/Passive -> straight to background taskkill without interrupting.
!macro CheckIfAppIsRunning executablePath productName
  !define /! UniqueID ${__LINE__}
  nsis_tauri_utils::StrReplace "$(appRunning)" "{{product_name}}" "${productName}"
  Pop $R1
  nsis_tauri_utils::StrReplace "$(appRunningOkKill)" "{{product_name}}" "${productName}"
  Pop $R2
  nsis_tauri_utils::StrReplace "$(failedToKillApp)" "{{product_name}}" "${productName}"
  Pop $R3
  ; tasklist probe (exit != 0 = app not running)
  nsis_tauri_utils::ExecShell "cmd /c tasklist /nh /fi ""imagename eq ${executablePath}""" "hide" ""
  Pop $R0
  ${If} $0 != 0
    Goto a2a_appcheck_done_${UniqueID}
  ${EndIf}
  IfSilent a2a_kill_${UniqueID} 0
  ${IfThen} $PassiveMode != 1 ${|} MessageBox MB_OKCANCEL $R2 IDOK a2a_kill_${UniqueID} IDCANCEL a2a_cancel_${UniqueID} ${|}
  a2a_kill_${UniqueID}:
    nsis_tauri_utils::ExecShell "taskkill /F /IM ${executablePath} /T" "hide" ""
    Goto a2a_appcheck_done_${UniqueID}
  a2a_cancel_${UniqueID}:
    Abort $R3
  a2a_appcheck_done_${UniqueID}:
  !undef UniqueID
!macroend
!endif

!macro NSIS_HOOK_PREINSTALL
  !insertmacro CheckIfAppIsRunning "workbuddy-proxy-desktop.exe" "${PRODUCTNAME}"
!macroend
