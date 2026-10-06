; Independently distributed Sunshine GUI component installer.
; It does not create a second application: files always go to the existing
; Sunshine installation's canonical assets\gui directory.

Unicode true
RequestExecutionLevel admin
ManifestDPIAware true

!include "MUI2.nsh"
!include "LogicLib.nsh"
!include "FileFunc.nsh"
!include "x64.nsh"

!ifndef VERSION
  !define VERSION "0.0.0-dev"
!endif
!ifndef SOURCE_DIR
  !define SOURCE_DIR "staging"
!endif
!ifndef OUTPUT_DIR
  !define OUTPUT_DIR "dist"
!endif

Name "Sunshine GUI"
OutFile "${OUTPUT_DIR}\Sunshine-GUI-Setup-${VERSION}.exe"
Icon "..\icons\icon.ico"
BrandingText "Sunshine GUI ${VERSION}"
InstallButtonText "$(InstallButton)"
ShowInstDetails show

!define MUI_ABORTWARNING
!define MUI_WELCOMEPAGE_TITLE "$(WelcomeTitle)"
!define MUI_WELCOMEPAGE_TEXT "$(WelcomeText)"
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_LANGUAGE "English"

LangString WelcomeTitle ${LANG_ENGLISH} "Sunshine GUI component update"
LangString WelcomeText ${LANG_ENGLISH} "This installer updates the GUI in the existing Sunshine installation.$\r$\n$\r$\nIt does not install a second copy of Sunshine or change your autostart preference."
LangString InstallButton ${LANG_ENGLISH} "Update"
LangString SunshineNotFound ${LANG_ENGLISH} "No valid Sunshine installation was found. Install or repair Sunshine first."
LangString CoreIncompatible ${LANG_ENGLISH} "The installed Sunshine core is not running or does not support this GUI. Start Sunshine or update the complete Sunshine installation first. No files were changed."
LangString BackupFailed ${LANG_ENGLISH} "The current GUI could not be prepared for replacement. Sunshine was not changed."
LangString InstallFailed ${LANG_ENGLISH} "The new GUI could not be installed. The previous GUI has been restored."
LangString InstallingGui ${LANG_ENGLISH} "Updating Sunshine GUI in $GuiDir"

Var SunshineDir
Var GuiDir
Var GuiPath
Var StylusPluginPath
Var Parameters

Function FindSunshine
  StrCpy $SunshineDir ""

  SetRegView 64
  ReadRegStr $SunshineDir HKLM "SOFTWARE\AlkaidLab\Sunshine" "InstallDir"
  IfFileExists "$SunshineDir\sunshine.exe" found
  ReadRegStr $SunshineDir HKLM "SOFTWARE\LizardByte\Sunshine" "InstallDir"
  IfFileExists "$SunshineDir\sunshine.exe" found
  ReadRegStr $SunshineDir HKLM "SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\Sunshine" "InstallLocation"
  IfFileExists "$SunshineDir\sunshine.exe" found

  SetRegView 32
  ReadRegStr $SunshineDir HKLM "SOFTWARE\AlkaidLab\Sunshine" "InstallDir"
  IfFileExists "$SunshineDir\sunshine.exe" found
  ReadRegStr $SunshineDir HKLM "SOFTWARE\LizardByte\Sunshine" "InstallDir"
  IfFileExists "$SunshineDir\sunshine.exe" found
  ReadRegStr $SunshineDir HKLM "SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\Sunshine" "InstallLocation"
  IfFileExists "$SunshineDir\sunshine.exe" found

  ${If} ${RunningX64}
    StrCpy $SunshineDir "$PROGRAMFILES64\Sunshine"
  ${Else}
    StrCpy $SunshineDir "$PROGRAMFILES\Sunshine"
  ${EndIf}
  IfFileExists "$SunshineDir\sunshine.exe" found

  StrCpy $SunshineDir ""
  Return

found:
  StrCpy $GuiDir "$SunshineDir\assets\gui"
  StrCpy $GuiPath "$GuiDir\sunshine-gui.exe"
  StrCpy $StylusPluginPath "$GuiDir\alkaidlab-plugin-stylus.dll"
FunctionEnd

Function .onInit
  ; The existing in-app updater invokes Inno-style /VERYSILENT. Accept it as
  ; an alias for NSIS /S so changing installer technology stays compatible.
  ${GetParameters} $Parameters
  ClearErrors
  ${GetOptions} $Parameters "/VERYSILENT" $0
  IfErrors +2
  SetSilent silent

  Call FindSunshine
  StrCmp $SunshineDir "" sunshine_not_found
  Return

sunshine_not_found:
  IfSilent no_message
  MessageBox MB_OK|MB_ICONSTOP "$(SunshineNotFound)"
no_message:
  SetErrorLevel 2
  Quit
FunctionEnd

Section "Sunshine GUI" MainSection
  DetailPrint "$(InstallingGui)"

  ; Probe the running Core with the new GUI before stopping or replacing the
  ; current Agent. This validates the tray protocol instead of guessing from
  ; unrelated product version strings.
  InitPluginsDir
  SetOutPath "$PLUGINSDIR"
  File /oname=sunshine-gui-compat.exe "${SOURCE_DIR}\sunshine-gui.exe"
  nsExec::ExecToStack '"$PLUGINSDIR\sunshine-gui-compat.exe" --check-core-compatibility'
  Pop $0
  Pop $1
  StrCmp $0 "0" core_compatible
  IfSilent core_incompatible_silent
  MessageBox MB_OK|MB_ICONSTOP "$(CoreIncompatible)"
core_incompatible_silent:
  SetErrorLevel 5
  Abort

core_compatible:
  CreateDirectory "$GuiDir"

  ; Only the user-session GUI locks these files. Never stop SunshineService,
  ; sunshine.exe, or sunshinesvc.exe: an active stream must keep running.
  nsExec::ExecToLog '"$SYSDIR\taskkill.exe" /F /IM sunshine-gui.exe'

  Delete "$StylusPluginPath.previous"
  IfFileExists "$StylusPluginPath" 0 backup_gui
  ClearErrors
  Rename "$StylusPluginPath" "$StylusPluginPath.previous"
  IfErrors backup_failed

backup_gui:
  Delete "$GuiPath.previous"
  IfFileExists "$GuiPath" 0 install_new_gui
  ClearErrors
  Rename "$GuiPath" "$GuiPath.previous"
  IfErrors backup_failed

install_new_gui:
  SetOutPath "$GuiDir"
  ClearErrors
  File /oname=sunshine-gui.exe "${SOURCE_DIR}\sunshine-gui.exe"
  IfErrors install_failed
  File /oname=alkaidlab-plugin-stylus.dll "${SOURCE_DIR}\alkaidlab-plugin-stylus.dll"
  IfErrors install_failed
  File /nonfatal /oname=WebView2Loader.dll "${SOURCE_DIR}\WebView2Loader.dll"
  Delete "$GuiPath.previous"
  Delete "$StylusPluginPath.previous"

  ; Interactive installs restore the tray agent immediately. Silent installs
  ; are driven by the in-app updater, which restarts the GUI itself.
  IfSilent install_done
  ExecShell "open" "$WINDIR\explorer.exe" '$\"$GuiPath$\" --hidden' SW_SHOWNORMAL
install_done:
  SetErrorLevel 0
  Goto section_done

backup_failed:
  IfFileExists "$StylusPluginPath.previous" 0 +2
  Rename "$StylusPluginPath.previous" "$StylusPluginPath"
  IfSilent backup_failed_silent
  MessageBox MB_OK|MB_ICONSTOP "$(BackupFailed)"
backup_failed_silent:
  SetErrorLevel 3
  Abort

install_failed:
  Delete "$GuiPath"
  Delete "$StylusPluginPath"
  IfFileExists "$GuiPath.previous" 0 +2
  Rename "$GuiPath.previous" "$GuiPath"
  IfFileExists "$StylusPluginPath.previous" 0 +2
  Rename "$StylusPluginPath.previous" "$StylusPluginPath"
  IfSilent install_failed_silent
  MessageBox MB_OK|MB_ICONSTOP "$(InstallFailed)"
install_failed_silent:
  SetErrorLevel 4
  Abort

section_done:
SectionEnd
