import os
import glob

replacements = [
    ("Nexora - The Quantum Prism Logo", "Nexora Logo"),
    ("Nexora - The Quantum Prism Streaming-First Desktop Browser", "Nexora Streaming-First Desktop Browser"),
    ("• Powered by The Quantum Prism Core", "• Ultra-Lightweight Desktop Browser"),
    ("Nexora - The Quantum Prism", "Nexora"),
    ("Quantum Prism .ico", "Nexora official .ico"),
    ("show The Quantum Prism icon", "show the official Nexora icon"),
]

paths = glob.glob("**/*.md", recursive=True) + glob.glob("scripts/*.ps1") + glob.glob("src/**/*.rs", recursive=True)

for p in paths:
    if "target" in p:
        continue
    if not os.path.isfile(p):
        continue
    try:
        with open(p, "r", encoding="utf-8") as f:
            content = f.read()
        changed = False
        for old, new in replacements:
            if old in content:
                content = content.replace(old, new)
                changed = True
        if changed:
            with open(p, "w", encoding="utf-8") as f:
                f.write(content)
            print(f"Updated {p}")
    except Exception as e:
        print(f"Skipping {p}: {e}")

print("Branding text cleanup complete.")

