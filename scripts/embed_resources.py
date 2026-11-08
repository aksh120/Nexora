import os
import sys

try:
    from PyInstaller.utils.win32 import icon as picon
    from PyInstaller.utils.win32 import versioninfo as pver
except ImportError:
    print("PyInstaller win32 utilities not available")
    sys.exit(0)

def embed(exe_path, ico_path):
    exe_path = os.path.abspath(exe_path)
    ico_path = os.path.abspath(ico_path)
    if not os.path.exists(exe_path):
        return
    print(f"Embedding icon {ico_path} into {exe_path}...")
    picon.CopyIcons_FromIco(exe_path, [ico_path])

    version = (0, 1, 0, 0)
    version_str = "0.1.0"
    ffi = pver.FixedFileInfo(
        filevers=version,
        prodvers=version,
        mask=0x3F,
        flags=0x0,
        OS=0x40004,
        fileType=0x1,
        subtype=0x0,
        date=(0, 0)
    )
    sfi = pver.StringFileInfo([
        pver.StringTable('040904b0', [
            pver.StringStruct('CompanyName', 'Nexora'),
            pver.StringStruct('FileDescription', 'Nexora'),
            pver.StringStruct('FileVersion', version_str),
            pver.StringStruct('InternalName', 'Nexora'),
            pver.StringStruct('LegalCopyright', 'Copyright (c) 2026 Nexora'),
            pver.StringStruct('OriginalFilename', 'Nexora.exe'),
            pver.StringStruct('ProductName', 'Nexora'),
            pver.StringStruct('ProductVersion', version_str),
        ])
    ])
    vfi = pver.VarFileInfo([pver.VarStruct('Translation', [1033, 1200])])
    vi = pver.VSVersionInfo(ffi=ffi, kids=[sfi, vfi])

    print(f"Writing version info into {exe_path}...")
    pver.write_version_info_to_executable(exe_path, vi)
    print(f"Successfully branded {exe_path} with Nexora icon and metadata!")

if __name__ == '__main__':
    project_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    ico = os.path.join(project_root, 'assets', 'branding', 'icon.ico')
    targets = [
        os.path.join(project_root, 'target', 'debug', 'Nexora.exe'),
        os.path.join(project_root, 'target', 'release', 'Nexora.exe'),
        os.path.join(project_root, 'dist', 'Nexora.exe'),
    ]
    for t in targets:
        if os.path.exists(t):
            embed(t, ico)

