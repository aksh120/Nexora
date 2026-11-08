import os

project_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
b64_path = os.path.join(project_root, 'assets', 'branding', 'app_icon_b64.txt')

with open(b64_path, 'r', encoding='utf-8') as f:
    b64 = f.read().strip()

icon_svg = f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256">
    <image href="data:image/png;base64,{b64}" width="256" height="256" />
</svg>'''

icon_path = os.path.join(project_root, 'assets', 'branding', 'icon.svg')
with open(icon_path, 'w', encoding='utf-8') as f:
    f.write(icon_svg)

logo_svg = f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 440 100" width="440" height="100" fill="none">
    <image href="data:image/png;base64,{b64}" x="10" y="5" width="90" height="90" />
    <text x="115" y="62" font-family="-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif" font-size="48" font-weight="700" letter-spacing="1.5" fill="#F4F4F5">
        NEX<tspan fill="#E11D48">O</tspan>RA
    </text>
    <text x="117" y="82" font-family="-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif" font-size="11" font-weight="500" letter-spacing="3" fill="#A1A1AA">
        STREAMING-FIRST BROWSER
    </text>
</svg>'''

logo_path = os.path.join(project_root, 'assets', 'branding', 'logo.svg')
with open(logo_path, 'w', encoding='utf-8') as f:
    f.write(logo_svg)

print("Updated assets/branding/icon.svg and assets/branding/logo.svg successfully!")
