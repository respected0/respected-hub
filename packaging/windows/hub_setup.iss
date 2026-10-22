; Respected Hub - per-user Windows installer (no administrator rights).
; Built by release/tools/build/build_hub_installer.py, which stages the Hub executable. Products are not inside: each product has
; its own installer, which the Hub downloads from the signed catalog and runs. Removing the Hub never touches a product.
#ifndef AppVersion
  #define AppVersion "0.1.0-beta.1"
#endif
#ifndef StageDir
  #define StageDir "..\..\..\.build-cache\hub-stage"
#endif
#ifndef OutputDir
  #define OutputDir "..\..\..\dist"
#endif

[Setup]
AppId={{5C0F6A1E-3B7D-4E2A-9D55-7A1C2F0E8B31}
AppName=Respected Hub
AppVersion={#AppVersion}
AppPublisher=Respected
AppCopyright=Copyright (c) 2026 RESPECTED (MIT)
DefaultDirName={localappdata}\Respected\RespectedHub\app
UsePreviousAppDir=yes
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
OutputDir={#OutputDir}
OutputBaseFilename=RespectedHub-Windows-Setup
Compression=lzma2
SolidCompression=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
UninstallDisplayIcon={app}\respected-hub.exe
CloseApplications=yes
SetupLogging=yes
WizardStyle=modern
InfoBeforeFile={#StageDir}\BETA_NOTICE.txt

[Languages]
Name: "turkish"; MessagesFile: "compiler:Languages\Turkish.isl"

[UninstallDelete]
; everything the Hub keeps is its own: the window data, the download cache and its activity journal
Type: filesandordirs; Name: "{localappdata}\com.respected.hub"
Type: filesandordirs; Name: "{userappdata}\com.respected.hub"
Type: filesandordirs; Name: "{localappdata}\Respected\RespectedHub"
Type: dirifempty; Name: "{localappdata}\Respected"

[Files]
Source: "{#StageDir}\respected-hub.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#StageDir}\dev-trust.pub"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist

[Icons]
Name: "{autoprograms}\Respected Hub"; Filename: "{app}\respected-hub.exe"
Name: "{autodesktop}\Respected Hub"; Filename: "{app}\respected-hub.exe"

[Run]
Filename: "{app}\respected-hub.exe"; Description: "Respected Hub'ı başlat"; Flags: nowait postinstall skipifsilent
; the Hub's own update runs this installer with /SILENT /RELAUNCH=1: open the Hub again afterwards
Filename: "{app}\respected-hub.exe"; Flags: nowait; Check: RelaunchAfterUpdate

[Code]
function WebView2Present(): Boolean;
var
  Version: String;
begin
  Result := (RegQueryStringValue(HKLM32, 'SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}', 'pv', Version) or
             RegQueryStringValue(HKCU, 'Software\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}', 'pv', Version)) and
            (Version <> '') and (Version <> '0.0.0.0');
end;

function InitializeSetup(): Boolean;
begin
  Result := True;
  // The Hub window cannot open without WebView2. Nothing is downloaded here: tell the person and let them decide.
  if not WebView2Present() and not WizardSilent then
    Result := MsgBox('Microsoft Edge WebView2 bulunamadı; Hub arayüzü onsuz açılmaz (Windows 11''de hazır gelir).' + #13#10 +
      'Kurulum devam edebilir, ancak WebView2 kurulana kadar Hub açılmaz (microsoft.com/edge/webview2).' + #13#10 + #13#10 +
      'Yine de devam edilsin mi?', mbConfirmation, MB_YESNO) = IDYES;
end;

function RelaunchAfterUpdate(): Boolean;
begin
  Result := ExpandConstant('{param:RELAUNCH|0}') = '1';
end;
