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
AppVerName=Respected Hub
UninstallDisplayName=Respected Hub
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
SetupIconFile=..\..\src-tauri\icons\icon.ico
CloseApplications=yes
SetupLogging=yes
WizardStyle=modern
WizardImageFile=wizard\side-164x314.bmp,wizard\side-192x366.bmp,wizard\side-246x470.bmp,wizard\side-328x628.bmp
DisableWelcomePage=no
InfoAfterFile=..\..\release\UPDATE_NOTES.md
InfoBeforeFile={#StageDir}\BETA_NOTICE.txt

[Languages]
Name: "turkish"; MessagesFile: "compiler:Languages\Turkish.isl"

[Messages]
turkish.WizardInfoBefore=Beta sürümü hakkında
turkish.InfoBeforeLabel=Bu kurucu henüz imzalı değil. Kurmadan önce aşağıdaki bilgileri oku.
turkish.WizardInfoAfter=Yenilikler
turkish.InfoAfterLabel=Bu sürümde neler değişti?
turkish.InfoAfterClickLabel=Devam etmek için Sonraki'ne tıklayın.
turkish.WelcomeLabel1=Respected Hub'a hoş geldin
turkish.WelcomeLabel2=Respected Hub, Brain ve Orchestrator'ı tek yerden kurar, günceller ve ayarlar.%n%nYönetici izni gerekmez; ürünlerin ve notlarının kurulumdan etkilenmez.%n%nDevam etmek için Sonraki'ne tıkla.
turkish.FinishedHeadingLabel=Respected Hub hazır
turkish.FinishedLabelNoIcons=Kurulum tamamlandı.
turkish.FinishedLabel=Kurulum tamamlandı. Hub'ı Başlat menüsünden ya da masaüstü kısayolundan açabilirsin.
; the ready page's button is "Kur", not "Sonraki"
turkish.ReadyLabel2a=Kuruluma devam etmek için Kur düğmesine, ayarları gözden geçirmek ya da değiştirmek için Önceki düğmesine tıklayın.
turkish.ReadyLabel2b=Kuruluma başlamak için Kur düğmesine tıklayın.

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

{ ---- already installed: repair / update or remove (the Hub's own silent update skips this page) ---- }

const
  UninstallKey = 'Software\Microsoft\Windows\CurrentVersion\Uninstall\{5C0F6A1E-3B7D-4E2A-9D55-7A1C2F0E8B31}_is1';

#include "respected_cards.iss"

var
  MaintenancePage: TWizardPage;
  Leaving: Boolean;

function InstalledVersion(): String;
begin
  if not RegQueryStringValue(HKCU, UninstallKey, 'DisplayVersion', Result) then Result := '';
end;

procedure InitializeWizard;
begin
  TidyInfoPages;
  if InstalledVersion() = '' then Exit;
  MaintenancePage := NewCardPage(0, wpWelcome, 'Respected Hub kurulu',
    'Kurulu sürüm v' + InstalledVersion() + ', bu kurucu v{#AppVersion}. Ürünlerine ve kasana dokunulmaz.', False);
  AddCard(MaintenancePage, 0, #$E895, 'Onar / Güncelle', 'Hub''ı bu kurucudaki sürümle (v{#AppVersion}) yeniler.', True);
  AddCard(MaintenancePage, 0, #$E74D, 'Kaldır', 'Hub kaldırılır; kurduğun ürünler ve notların yerinde kalır.', False);
end;

procedure ProductPageChanged(CurPageID: Integer);
begin
end;

function ShouldSkipPage(PageID: Integer): Boolean;
begin
  // the what's-new page is only for an update over an installed Hub
  Result := ((MaintenancePage <> nil) and (PageID = MaintenancePage.ID) and WizardSilent) or ((PageID = wpInfoAfter) and (MaintenancePage = nil));
end;

procedure CancelButtonClick(CurPageID: Integer; var Cancel, Confirm: Boolean);
begin
  if Leaving then Confirm := False;
end;

function NextButtonClick(CurPageID: Integer): Boolean;
var
  Uninstaller: String;
  Code: Integer;
begin
  Result := True;
  if (MaintenancePage = nil) or (CurPageID <> MaintenancePage.ID) or (CardSelectedIndex(0) <> 1) then Exit;
  if RegQueryStringValue(HKCU, UninstallKey, 'UninstallString', Uninstaller) then
    Exec(RemoveQuotes(Uninstaller), '', '', SW_SHOWNORMAL, ewNoWait, Code);
  Result := False;
  Leaving := True;
  WizardForm.Close;
end;
