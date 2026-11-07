; NEXORA Inno Setup Script
; Ultra-Lightweight, Streaming-First Desktop Browser for Windows 10 & 11
; Builds: Nexora-Setup-v0.1.0.exe

#define MyAppName "Nexora"
#define MyAppVersion "0.1.0"
#define MyAppPublisher "Nexora Development Team"
#define MyAppURL "https://github.com/Nexora/Nexora"
#define MyAppExeName "Nexora.exe"

[Setup]
; Unique AppId generated for Nexora
AppId={{5E7A23F1-C349-41C2-82DF-3BC177D5716E}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
LicenseFile=..\LICENSE
OutputDir=..\dist
OutputBaseFilename=Nexora-Setup-v{#MyAppVersion}
SetupIconFile=..\assets\branding\icon.ico
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0.17763
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "startmenuicon"; Description: "Create Start Menu shortcut"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
; Main executable compiled in release profile
Source: "..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
; Visual identity assets
Source: "..\assets\branding\*"; DestDir: "{app}\assets\branding"; Flags: ignoreversion recursesubdirs createallsubdirs
; Core licenses and docs
Source: "..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Registry]
; Register App Paths for Windows Run dialog (Win+R -> nexora)
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\App Paths\{#MyAppExeName}"; ValueType: string; ValueName: ""; ValueData: "{app}\{#MyAppExeName}"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\App Paths\{#MyAppExeName}"; ValueType: string; ValueName: "Path"; ValueData: "{app}"; Flags: uninsdeletekey

; Register Nexora URL Scheme (nexora://)
Root: HKCU; Subkey: "Software\Classes\nexora"; ValueType: string; ValueName: ""; ValueData: "URL:Nexora Protocol"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\nexora"; ValueType: string; ValueName: "URL Protocol"; ValueData: ""; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\nexora\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\{#MyAppExeName},0"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\nexora\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Flags: uninsdeletekey

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent

[Code]
// Helper function to check if WebView2 Evergreen Runtime is installed
function IsWebView2Installed(): Boolean;
var
  InstalledVersion: String;
begin
  Result := RegQueryStringValue(HKLM, 'SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-9E47-44E9-B977-9E4935400D2B}', 'pv', InstalledVersion) or
            RegQueryStringValue(HKCU, 'SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-9E47-44E9-B977-9E4935400D2B}', 'pv', InstalledVersion);
  if (Result and (Length(InstalledVersion) > 0)) then
    Log('Detected WebView2 Evergreen Runtime: ' + InstalledVersion)
  else
    Log('WebView2 Evergreen Runtime was not detected on this system.');
end;

function InitializeSetup(): Boolean;
var
  ErrorCode: Integer;
begin
  Result := True;
  if not IsWebView2Installed() then
  begin
    if MsgBox('NEXORA requires the Microsoft Edge WebView2 Evergreen Runtime, which is not currently detected.' + #13#10 + #13#10 +
              'Would you like to open the Microsoft WebView2 download page now?', mbConfirmation, MB_YESNO) = IDYES then
    begin
      ShellExec('open', 'https://go.microsoft.com/fwlink/p/?LinkId=2124703', '', '', SW_SHOWNORMAL, ewNoWait, ErrorCode);
    end;
  end;
end;
