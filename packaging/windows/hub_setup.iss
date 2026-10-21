; Respected Hub - per-user Windows installer (no administrator rights).
; Built by release/tools/build/build_hub_installer.py, which stages the Hub executable. Products are not inside: the Hub downloads
; them from the signed catalog on GitHub Releases when the person installs them.
#ifndef AppVersion
  #define AppVersion "0.1.0-beta.1"
#endif
#ifndef Product
  #define Product ""
#endif
#ifndef OutputBase
  #define OutputBase "RespectedHub-Windows-Setup"
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
DefaultDirName={localappdata}\Respected\RespectedHub\app
UsePreviousAppDir=yes
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
OutputDir={#OutputDir}
OutputBaseFilename={#OutputBase}
Compression=lzma2
SolidCompression=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
UninstallDisplayIcon={app}\respected-hub.exe
CloseApplications=yes
WizardStyle=modern
InfoBeforeFile={#StageDir}\BETA_NOTICE.txt

[InstallDelete]
; installers before 0.1.0-beta.1 shipped a product catalog here; products now come from the downloaded catalog cache
Type: filesandordirs; Name: "{app}\catalog"

[UninstallDelete]
; the WebView cache and window settings are the Hub's own; the products (managed, data) are removed only through --uninstall-products
Type: filesandordirs; Name: "{localappdata}\com.respected.hub\EBWebView"
Type: files; Name: "{localappdata}\Respected\RespectedHub\uninstall-products.log"
Type: dirifempty; Name: "{localappdata}\com.respected.hub"
Type: filesandordirs; Name: "{userappdata}\com.respected.hub"
Type: dirifempty; Name: "{localappdata}\Respected\RespectedHub"
Type: dirifempty; Name: "{localappdata}\Respected"

[Files]
Source: "{#StageDir}\respected-hub.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#StageDir}\dev-trust.pub"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist

[Icons]
Name: "{autoprograms}\Respected Hub"; Filename: "{app}\respected-hub.exe"
Name: "{autodesktop}\Respected Hub"; Filename: "{app}\respected-hub.exe"

[Run]
Filename: "{app}\respected-hub.exe"; Parameters: "{#Product}"; Description: "Respected Hub'ı başlat"; Flags: nowait postinstall skipifsilent
; the Hub's own update runs this installer with /SILENT /RELAUNCH=1: open the Hub again afterwards
Filename: "{app}\respected-hub.exe"; Flags: nowait; Check: RelaunchAfterUpdate

[Code]
var
  ProductsAsked, ProductsWanted: Boolean;

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

// The Hub lists each product it installed in Installed apps (src-tauri/src/hub/registration.rs): that is how we know any exist.
function ProductsInstalled(): Boolean;
begin
  Result := RegKeyExists(HKCU, 'Software\Microsoft\Windows\CurrentVersion\Uninstall\RespectedSecondBrain') or
            RegKeyExists(HKCU, 'Software\Microsoft\Windows\CurrentVersion\Uninstall\RespectedOrchestrator');
end;

function RemoveProducts(): Boolean;
begin
  if not ProductsAsked then
  begin
    ProductsAsked := True;
    if ExpandConstant('{param:REMOVEPRODUCTS|0}') = '1' then
      ProductsWanted := True
    else if not UninstallSilent and ProductsInstalled() then
      ProductsWanted := MsgBox('Hub ile kurulan SecondBrain ve Orchestrator da kaldırılsın mı?' + #13#10 + #13#10 +
        'Evet: ürünler, verileri ve AI araçlarındaki Respected kayıtları (kanca, MCP, görev, kısayol) kaldırılır. Not kasanız ve notlarınız silinmez.' + #13#10 + #13#10 +
        'Hayır: ürünler çalışmaya devam eder ama Hub olmadan güncellenemez ve onarılamaz. Onları sonra Windows Ayarlar > Uygulamalar listesinden kaldırabilirsiniz; bunun için küçük bir bakım dosyası kalır.',
        mbConfirmation, MB_YESNO or MB_DEFBUTTON2) = IDYES;
  end;
  Result := ProductsWanted;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  ResultCode: Integer;
  Maintenance: String;
begin
  // Before the Hub's own files are removed. (Decided here, not with an [UninstallRun] Check: Inno evaluates that Check at install time.)
  if CurUninstallStep = usUninstall then
  begin
    if RemoveProducts() then
      // undo the AI-client registrations of the products the Hub installed and remove them
      Exec(ExpandConstant('{app}\respected-hub.exe'), '--uninstall-products', '', SW_HIDE, ewWaitUntilTerminated, ResultCode)
    else if ProductsInstalled() then
    begin
      // products kept: their Installed-apps entries must point at an executable that survives this uninstall
      Maintenance := ExpandConstant('{localappdata}\Respected\RespectedHub\maintenance');
      if ForceDirectories(Maintenance) and CopyFile(ExpandConstant('{app}\respected-hub.exe'), Maintenance + '\respected-hub.exe', False) then
        Exec(Maintenance + '\respected-hub.exe', '--sync-registrations', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
    end;
  end;
end;

// Uninstall removes what the installer put here and, only when asked, the products the Hub installed (see above).
// Note vaults live elsewhere and are never touched.
