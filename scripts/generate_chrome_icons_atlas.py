import os
from PyQt6.QtSvg import QSvgRenderer
from PyQt6.QtGui import QPainter, QImage, QColor
from PyQt6.QtCore import QByteArray, QSize, Qt, QRectF
from PIL import Image
import numpy as np

# 2X High-DPI Supersampled Icon slot size (48x48)
ICON_SIZE = 48

# Color tokens:
COLOR_NORMAL = "#E4E4E7"     # High-contrast soft white (Zinc-200)
COLOR_HOVER = "#FFFFFF"      # Pure white
COLOR_ACCENT = "#E11D48"     # Eclipse Crimson
COLOR_ACCENT_HOVER = "#F43F5E" # Rose-500
COLOR_MUTED = "#71717A"      # Zinc-500 (inactive/disabled)
COLOR_SHIELD = "#10B981"     # Emerald-500

STROKE_W = 3.2

# List of icons in atlas:
# (Index, Name, SVG markup, stroke_color, fill_color, stroke_width)
lucide_defs = [
    # 0: Official Brand Logo (special handling from app_icon.png)
    ("logo", None, None, None, 0),
    
    # 1: Back normal
    ("back_normal", '<path d="m12 19-7-7 7-7"/><path d="M19 12H5"/>', COLOR_NORMAL, "none", STROKE_W),
    # 2: Back hover
    ("back_hover", '<path d="m12 19-7-7 7-7"/><path d="M19 12H5"/>', COLOR_HOVER, "none", STROKE_W),
    
    # 3: Forward normal
    ("fwd_normal", '<path d="M5 12h14"/><path d="m12 5 7 7-7 7"/>', COLOR_NORMAL, "none", STROKE_W),
    # 4: Forward hover
    ("fwd_hover", '<path d="M5 12h14"/><path d="m12 5 7 7-7 7"/>', COLOR_HOVER, "none", STROKE_W),
    
    # 5: Reload normal
    ("reload_normal", '<path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8"/><path d="M21 3v5h-5"/>', COLOR_NORMAL, "none", STROKE_W),
    # 6: Reload hover
    ("reload_hover", '<path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8"/><path d="M21 3v5h-5"/>', COLOR_HOVER, "none", STROKE_W),
    
    # 7: Home normal
    ("home_normal", '<path d="M15 21v-8a1 1 0 0 0-1-1h-4a1 1 0 0 0-1 1v8"/><path d="M3 10a2 2 0 0 1 .709-1.528l7-5.999a2 2 0 0 1 2.582 0l7 5.999A2 2 0 0 1 21 10v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>', COLOR_NORMAL, "none", STROKE_W),
    # 8: Home hover
    ("home_hover", '<path d="M15 21v-8a1 1 0 0 0-1-1h-4a1 1 0 0 0-1 1v8"/><path d="M3 10a2 2 0 0 1 .709-1.528l7-5.999a2 2 0 0 1 2.582 0l7 5.999A2 2 0 0 1 21 10v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>', COLOR_HOVER, "none", STROKE_W),
    
    # 9: ShieldCheck (emerald security)
    ("shield_check", '<path d="M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z"/><path d="m9 12 2 2 4-4"/>', COLOR_SHIELD, "none", STROKE_W),
    
    # 10: Bookmark Outline normal
    ("bm_outline_normal", '<path d="m19 21-7-4-7 4V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2v16z"/>', COLOR_NORMAL, "none", STROKE_W),
    # 11: Bookmark Outline hover
    ("bm_outline_hover", '<path d="m19 21-7-4-7 4V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2v16z"/>', COLOR_ACCENT, "none", STROKE_W),
    # 12: Bookmark Filled
    ("bm_filled_normal", '<path d="m19 21-7-4-7 4V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2v16z"/>', COLOR_ACCENT, COLOR_ACCENT, STROKE_W),
    # 13: Bookmark Filled hover
    ("bm_filled_hover", '<path d="m19 21-7-4-7 4V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2v16z"/>', COLOR_ACCENT_HOVER, COLOR_ACCENT_HOVER, STROKE_W),
    
    # 14: Menu normal
    ("menu_normal", '<line x1="4" x2="20" y1="12" y2="12"/><line x1="4" x2="20" y1="6" y2="6"/><line x1="4" x2="20" y1="18" y2="18"/>', COLOR_NORMAL, "none", STROKE_W),
    # 15: Menu hover
    ("menu_hover", '<line x1="4" x2="20" y1="12" y2="12"/><line x1="4" x2="20" y1="6" y2="6"/><line x1="4" x2="20" y1="18" y2="18"/>', COLOR_HOVER, "none", STROKE_W),
    
    # 16: Plus (New tab) normal
    ("plus_normal", '<path d="M5 12h14"/><path d="M12 5v14"/>', COLOR_NORMAL, "none", STROKE_W),
    # 17: Plus hover
    ("plus_hover", '<path d="M5 12h14"/><path d="M12 5v14"/>', COLOR_ACCENT, "none", STROKE_W),
    
    # 18: Close (X) normal (tab inactive)
    ("close_normal", '<path d="M18 6 6 18"/><path d="m6 6 12 12"/>', COLOR_MUTED, "none", STROKE_W),
    # 19: Close (X) active tab
    ("close_active", '<path d="M18 6 6 18"/><path d="m6 6 12 12"/>', COLOR_NORMAL, "none", STROKE_W),
    # 20: Close (X) hover
    ("close_hover", '<path d="M18 6 6 18"/><path d="m6 6 12 12"/>', COLOR_ACCENT, "none", STROKE_W),

    # 21: Caption Minimize normal
    ("cap_min_normal", '<line x1="4" x2="20" y1="12" y2="12"/>', COLOR_NORMAL, "none", STROKE_W),
    # 22: Caption Minimize hover
    ("cap_min_hover", '<line x1="4" x2="20" y1="12" y2="12"/>', COLOR_HOVER, "none", STROKE_W),
    # 23: Caption Maximize normal
    ("cap_max_normal", '<rect x="4" y="4" width="16" height="16" rx="2"/>', COLOR_NORMAL, "none", STROKE_W),
    # 24: Caption Maximize hover
    ("cap_max_hover", '<rect x="4" y="4" width="16" height="16" rx="2"/>', COLOR_HOVER, "none", STROKE_W),
    # 25: Caption Restore normal
    ("cap_restore_normal", '<path d="M8 4h12v12"/><rect x="4" y="8" width="12" height="12" rx="2"/>', COLOR_NORMAL, "none", STROKE_W),
    # 26: Caption Restore hover
    ("cap_restore_hover", '<path d="M8 4h12v12"/><rect x="4" y="8" width="12" height="12" rx="2"/>', COLOR_HOVER, "none", STROKE_W),
    # 27: Caption Close normal
    ("cap_close_normal", '<path d="M18 6 6 18"/><path d="m6 6 12 12"/>', COLOR_NORMAL, "none", STROKE_W),
    # 28: Caption Close hover
    ("cap_close_hover", '<path d="M18 6 6 18"/><path d="m6 6 12 12"/>', COLOR_HOVER, "none", STROKE_W),

    # 29: Forward disabled (muted)
    ("fwd_disabled", '<path d="M5 12h14"/><path d="m12 5 7 7-7 7"/>', COLOR_MUTED, "none", STROKE_W),
    # 30: Back disabled (muted)
    ("back_disabled", '<path d="m12 19-7-7 7-7"/><path d="M19 12H5"/>', COLOR_MUTED, "none", STROKE_W),

    # 31: Volume2 normal (Audio playing)
    ("volume2_normal", '<polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/><path d="M15.54 8.46a5 5 0 0 1 0 7.07"/><path d="M19.07 4.93a10 10 0 0 1 0 14.14"/>', COLOR_NORMAL, "none", STROKE_W),
    # 32: Volume2 hover
    ("volume2_hover", '<polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/><path d="M15.54 8.46a5 5 0 0 1 0 7.07"/><path d="M19.07 4.93a10 10 0 0 1 0 14.14"/>', COLOR_HOVER, "none", STROKE_W),

    # 33: VolumeX normal (Muted)
    ("volumex_normal", '<polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/><line x1="22" x2="16" y1="9" y2="15"/><line x1="16" x2="22" y1="9" y2="15"/>', COLOR_ACCENT, "none", STROKE_W),
    # 34: VolumeX hover
    ("volumex_hover", '<polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/><line x1="22" x2="16" y1="9" y2="15"/><line x1="16" x2="22" y1="9" y2="15"/>', COLOR_ACCENT_HOVER, "none", STROKE_W),

    # 35: Pin normal
    ("pin_normal", '<line x1="12" x2="12" y1="17" y2="22"/><path d="M5 17h14v-1.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V6h1a2 2 0 0 0 0-4H8a2 2 0 0 0 0 4h1v4.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24Z"/>', COLOR_NORMAL, "none", STROKE_W),
    # 36: Pin hover
    ("pin_hover", '<line x1="12" x2="12" y1="17" y2="22"/><path d="M5 17h14v-1.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V6h1a2 2 0 0 0 0-4H8a2 2 0 0 0 0 4h1v4.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24Z"/>', COLOR_ACCENT, "none", STROKE_W),

    # 37: ChevronUp (Find previous)
    ("chevron_up", '<path d="m18 15-6-6-6 6"/>', COLOR_NORMAL, "none", STROKE_W),
    # 38: ChevronDown (Find next)
    ("chevron_down", '<path d="m6 9 6 6 6-6"/>', COLOR_NORMAL, "none", STROKE_W),

    # 39: Search (Find in Page)
    ("search_icon", '<circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/>', COLOR_MUTED, "none", STROKE_W),
]

num_icons = len(lucide_defs)
atlas_w = num_icons * ICON_SIZE
atlas_h = ICON_SIZE

# Create QImage in ARGB32_Premultiplied
atlas_qimg = QImage(QSize(atlas_w, atlas_h), QImage.Format.Format_ARGB32_Premultiplied)
atlas_qimg.fill(QColor(0, 0, 0, 0))

painter = QPainter(atlas_qimg)
painter.setRenderHint(QPainter.RenderHint.Antialiasing, True)
painter.setRenderHint(QPainter.RenderHint.SmoothPixmapTransform, True)

# Draw Logo at index 0 from assets/branding/app_icon.png
app_icon_path = os.path.abspath('assets/branding/app_icon.png')
if os.path.exists(app_icon_path):
    logo_pil = Image.open(app_icon_path)
    logo_pil_resized = logo_pil.resize((ICON_SIZE - 4, ICON_SIZE - 4), Image.Resampling.LANCZOS)
    logo_rgba = np.array(logo_pil_resized)
    logo_qimg = QImage(logo_rgba.data, logo_pil_resized.width, logo_pil_resized.height, QImage.Format.Format_RGBA8888)
    painter.drawImage(2, 2, logo_qimg)

# Draw Lucide icons
# In 48x48 box, 4px inset gives 40x40 icon area (matches optical fullness)
for i in range(1, num_icons):
    name, path_data, stroke, fill, stroke_w = lucide_defs[i]
    svg = f'<svg xmlns="http://www.w3.org/2000/svg" width="{ICON_SIZE}" height="{ICON_SIZE}" viewBox="0 0 24 24" fill="{fill}" stroke="{stroke}" stroke-width="{stroke_w}" stroke-linecap="round" stroke-linejoin="round">{path_data}</svg>'
    renderer = QSvgRenderer(QByteArray(svg.encode('utf-8')))
    x_offset = i * ICON_SIZE
    renderer.render(painter, QRectF(x_offset + 5.0, 5.0, 38.0, 38.0))

painter.end()

out_png = os.path.abspath('assets/branding/chrome_icons_atlas.png')
atlas_qimg.save(out_png)

# Dump raw premultiplied BGRA bytes
ptr = atlas_qimg.bits()
ptr.setsize(atlas_w * atlas_h * 4)
raw_bytes = bytes(ptr)

out_bin = os.path.abspath('assets/branding/chrome_icons_atlas.bin')
with open(out_bin, 'wb') as f:
    f.write(raw_bytes)

# Also copy to dist folder
dist_assets = os.path.abspath('dist/Nexora-v0.1.0-portable/assets/branding')
if os.path.exists(dist_assets):
    atlas_qimg.save(os.path.join(dist_assets, 'chrome_icons_atlas.png'))
    with open(os.path.join(dist_assets, 'chrome_icons_atlas.bin'), 'wb') as f:
        f.write(raw_bytes)

print(f"High-Res Icon atlas generated! Icons: {num_icons}, Size: {atlas_w}x{atlas_h}, Bytes: {len(raw_bytes)}")
