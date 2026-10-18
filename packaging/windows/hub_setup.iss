; Respected Hub - per-user Windows installer (no administrator rights).
; Built by tools/build_hub_installer.py, which stages the Hub executable and the product catalog first.
#ifndef AppVersion
  #define AppVersion "0.1.0"
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
; the shipped catalog is replaced as a whole so bundles of an older/newer build never linger and look like updates
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
Source: "{#StageDir}\catalog\*"; DestDir: "{app}\catalog"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{autoprograms}\Respected Hub"; Filename: "{app}\respected-hub.exe"
Name: "{autodesktop}\Respected Hub"; Filename: "{app}\respected-hub.exe"

[Run]
Filename: "{app}\respected-hub.exe"; Parameters: "{#Product}"; Description: "Respected Hub'ı başlat"; Flags: nowait postinstall skipifsilent

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

function RemoveProducts(): Boolean;
begin
  if not ProductsAsked then
  begin
    ProductsAsked := True;
    if ExpandConstant('{param:REMOVEPRODUCTS|0}') = '1' then
      ProductsWanted := True
    else if not UninstallSilent then
      ProductsWanted := MsgBox('Hub ile kurulan SecondBrain ve Orchestrator da kaldırılsın mı?' + #13#10 + #13#10 +
        'Evet: AI araçlarındaki kancalar ve MCP kayıtları geri alınır, ürünler kaldırılır. Notlarınız, kasanız ve çalışma kayıtlarınız silinmez.' + #13#10 +
        'Hayır: ürünler çalışmaya devam eder, ancak Hub olmadan yönetilemez.', mbConfirmation, MB_YESNO) = IDYES;
  end;
  Result := ProductsWanted;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  ResultCode: Integer;
begin
  // Before the Hub's own files are removed: undo the AI-client registrations of the products the Hub installed and remove them.
  // (Decided here, not with an [UninstallRun] Check: Inno evaluates that Check at install time.)
  if (CurUninstallStep = usUninstall) and RemoveProducts() then
    Exec(ExpandConstant('{app}\respected-hub.exe'), '--uninstall-products', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
end;

// Uninstall removes what the installer put here and, only when asked, the products the Hub installed (see above).
// Notes, vaults and Orchestrator run data live elsewhere and are not touched.
