!define PRODUCT_NAME "ZeroClash"
!define PRODUCT_VERSION "REPLACE_VERSION"
!define PRODUCT_PUBLISHER "ZeroClash Labs"
!define PRODUCT_WEB_SITE "https://github.com/zeroclash-labs/zeroclash"
!define PRODUCT_DIR_REGKEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}"
!define PRODUCT_UNINST_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}"
!define PRODUCT_UNINST_ROOT_KEY "HKLM"

SetCompressor lzma

!include "MUI.nsh"
!include "LogicLib.nsh"
!include "FileFunc.nsh"

!define MUI_ABORTWARNING

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "LICENSE.txt"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"

Name "${PRODUCT_NAME} ${PRODUCT_VERSION}"
OutFile "ZeroClash-${PRODUCT_VERSION}-setup.exe"
InstallDir "$PROGRAMFILES64\ZeroClash"
InstallDirRegKey HKLM "${PRODUCT_DIR_REGKEY}" ""
ShowInstDetails show
ShowUnInstDetails show
RequestExecutionLevel admin

Section "MainSection" SEC01
    SetOutPath "$INSTDIR"
    SetOverwrite ifnewer
    File "target\REPLACE_TARGET\release\zeroclash.exe"
    File "target\REPLACE_TARGET\release\zeroclash-cli.exe"
    File "target\REPLACE_TARGET\release\mihomo.exe"
    CreateDirectory "$SMPROGRAMS\ZeroClash"
    CreateShortCut "$SMPROGRAMS\ZeroClash\ZeroClash.lnk" "$INSTDIR\zeroclash.exe"
    CreateShortCut "$DESKTOP\ZeroClash.lnk" "$INSTDIR\zeroclash.exe"
SectionEnd

Section -Post
    WriteUninstaller "$INSTDIR\uninst.exe"
    WriteRegStr HKLM "${PRODUCT_DIR_REGKEY}" "" "$INSTDIR\zeroclash.exe"
    WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" \
        "DisplayName" "$(^Name)"
    WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" \
        "UninstallString" "$INSTDIR\uninst.exe"
    WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" \
        "DisplayVersion" "${PRODUCT_VERSION}"
    WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" \
        "URLInfoAbout" "${PRODUCT_WEB_SITE}"
    WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" \
        "Publisher" "${PRODUCT_PUBLISHER}"
SectionEnd

Section Uninstall
    Delete "$INSTDIR\zeroclash.exe"
    Delete "$INSTDIR\zeroclash-cli.exe"
    Delete "$INSTDIR\mihomo.exe"
    Delete "$INSTDIR\uninst.exe"
    RMDir "$INSTDIR"
    Delete "$SMPROGRAMS\ZeroClash\ZeroClash.lnk"
    RMDir "$SMPROGRAMS\ZeroClash"
    Delete "$DESKTOP\ZeroClash.lnk"
    DeleteRegKey ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}"
    DeleteRegKey HKLM "${PRODUCT_DIR_REGKEY}"
    SetAutoClose true
SectionEnd
