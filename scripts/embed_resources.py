import os
import sys

try:
    from PyInstaller.utils.win32 import icon as picon
    from PyInstaller.utils.win32 import versioninfo as pver
except ImportError:
    print("PyInstaller win32 utilities not available")
    sys.exit(0)

def embed(exe_path, ico_path, is_installer=False):
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

    desc = "Nexora Setup" if is_installer else "Nexora"
    orig_name = "Nexora-Setup-v0.1.0.exe" if is_installer else "Nexora.exe"
    internal_name = "NexoraSetup" if is_installer else "Nexora"

    sfi = pver.StringFileInfo([
        pver.StringTable('040904b0', [
            pver.StringStruct('CompanyName', 'Akshat Apoorv'),
            pver.StringStruct('FileDescription', desc),
            pver.StringStruct('FileVersion', version_str),
            pver.StringStruct('InternalName', internal_name),
            pver.StringStruct('LegalCopyright', 'Copyright (c) 2026 Akshat Apoorv'),
            pver.StringStruct('OriginalFilename', orig_name),
            pver.StringStruct('ProductName', 'Nexora'),
            pver.StringStruct('ProductVersion', version_str),
        ])
    ])
    vfi = pver.VarFileInfo([pver.VarStruct('Translation', [1033, 1200])])
    vi = pver.VSVersionInfo(ffi=ffi, kids=[sfi, vfi])

    print(f"Writing version info ({desc}) into {exe_path}...")
    pver.write_version_info_to_executable(exe_path, vi)
    print(f"Successfully branded {exe_path} with Nexora icon and metadata!")

if __name__ == '__main__':
    project_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    ico = os.path.join(project_root, 'assets', 'branding', 'icon.ico')

    # Main browser binaries
    browser_targets = [
        os.path.join(project_root, 'Nexora.exe'),
        os.path.join(project_root, 'target', 'debug', 'Nexora.exe'),
        os.path.join(project_root, 'target', 'release', 'Nexora.exe'),
        os.path.join(project_root, 'dist', 'Nexora.exe'),
        os.path.join(project_root, 'dist', 'Nexora-v0.1.0-portable', 'Nexora.exe'),
    ]
    for t in browser_targets:
        if os.path.exists(t):
            embed(t, ico, is_installer=False)

    # Installer / Setup binaries
    installer_targets = [
        os.path.join(project_root, 'NexoraSetup.exe'),
        os.path.join(project_root, 'installer', 'target', 'release', 'NexoraSetup.exe'),
        os.path.join(project_root, 'dist', 'Nexora-Setup-v0.1.0.exe'),
        os.path.join(project_root, 'dist', 'NexoraSetup.exe'),
    ]
    for t in installer_targets:
        if os.path.exists(t):
            embed(t, ico, is_installer=True)
