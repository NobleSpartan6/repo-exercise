#ifndef AppVersion
  #error AppVersion must be supplied by scripts/package.py windows
#endif

[Setup]
AppId={{5C2B3BEE-49D9-44D4-9409-1875209B5F78}
AppName=Burrow
AppVersion={#AppVersion}
AppPublisher=Burrow contributors
AppPublisherURL=https://github.com/NobleSpartan6/burrow
DefaultDirName={localappdata}\Programs\Burrow
UsePreviousAppDir=yes
DefaultGroupName=Burrow
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
SourceDir=..
OutputDir=dist
OutputBaseFilename=Burrow-{#AppVersion}-Windows-x64-Setup
SetupIconFile=staging\icons\burrow.ico
UninstallDisplayIcon={app}\burrow.exe
LicenseFile=LICENSE
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
CloseApplications=yes
CloseApplicationsFilter=burrow.exe
RestartApplications=no

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; Flags: unchecked

[Files]
Source: "target\release\burrow.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "LICENSE"; DestDir: "{app}"; DestName: "LICENSE.txt"
Source: "THIRD_PARTY_NOTICES.txt"; DestDir: "{app}"
Source: "docs\INSTALL.md"; DestDir: "{app}"; DestName: "READ ME FIRST.txt"

[Icons]
Name: "{autoprograms}\Burrow"; Filename: "{app}\burrow.exe"
Name: "{autodesktop}\Burrow"; Filename: "{app}\burrow.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\burrow.exe"; Description: "Open Burrow"; Flags: nowait postinstall skipifsilent
