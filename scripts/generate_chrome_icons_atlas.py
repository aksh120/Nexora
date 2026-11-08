import os
from PyQt6.QtSvg import QSvgRenderer
from PyQt6.QtGui import QPainter, QImage, QColor
from PyQt6.QtCore import QByteArray, QSize, Qt
from PIL import Image
import numpy as np

# Icon slot size
ICON_SIZE = 24

# List of icons to render into atlas:
# (Index, Name, SVG markup, stroke_color, fill_color, stroke_width)
lucide_defs = [
    # 0: Official Brand Logo (special handling from app_icon.png)
    ("logo", None, None, None, 0),
    
    # 1: Back normal (#A1A1AA)
    ("back_normal", '<path d="m12 19-7-7 7-7"/><path d="M19 12H5"/>', "#A1A1AA", "none", 2.0),
    # 2: Back hover (#FFFFFF)
    ("back_hover", '<path d="m12 19-7-7 7-7"/><path d="M19 12H5"/>', "#FFFFFF", "none", 2.0),
    
    # 3: Forward normal (#A1A1AA)
    ("fwd_normal", '<path d="M5 12h14"/><path d="m12 5 7 7-7 7"/>', "#A1A1AA", "none", 2.0),
    # 4: Forward hover (#FFFFFF)
    ("fwd_hover", '<path d="M5 12h14"/><path d="m12 5 7 7-7 7"/>', "#FFFFFF", "none", 2.0),
    
    # 5: Reload normal (#A1A1AA)
    ("reload_normal", '<path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8"/><path d="M21 3v5h-5"/>', "#A1A1AA", "none", 2.0),
    # 6: Reload hover (#FFFFFF)
    ("reload_hover", '<path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8"/><path d="M21 3v5h-5"/>', "#FFFFFF", "none", 2.0),
    
    # 7: Home normal (#A1A1AA)
    ("home_normal", '<path d="M15 21v-8a1 1 0 0 0-1-1h-4a1 1 0 0 0-1 1v8"/><path d="M3 10a2 2 0 0 1 .709-1.528l7-5.999a2 2 0 0 1 2.582 0l7 5.999A2 2 0 0 1 21 10v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>', "#A1A1AA", "none", 2.0),
    # 8: Home hover (#FFFFFF)
    ("home_hover", '<path d="M15 21v-8a1 1 0 0 0-1-1h-4a1 1 0 0 0-1 1v8"/><path d="M3 10a2 2 0 0 1 .709-1.528l7-5.999a2 2 0 0 1 2.582 0l7 5.999A2 2 0 0 1 21 10v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>', "#FFFFFF", "none", 2.0),
    
    # 9: ShieldCheck (#10B981)
    ("shield_check", '<path d="M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z"/><path d="m9 12 2 2 4-4"/>', "#10B981", "none", 2.0),
    
    # 10: Bookmark Outline normal (#A1A1AA)
    ("bm_outline_normal", '<path d="m19 21-7-4-7 4V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2v16z"/>', "#A1A1AA", "none", 2.0),
    # 11: Bookmark Outline hover (#E11D48)
    ("bm_outline_hover", '<path d="m19 21-7-4-7 4V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2v16z"/>', "#E11D48", "none", 2.0),
    # 12: Bookmark Filled (#E11D48)
    ("bm_filled_normal", '<path d="m19 21-7-4-7 4V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2v16z"/>', "#E11D48", "#E11D48", 2.0),
    # 13: Bookmark Filled hover (#F43F5E)
    ("bm_filled_hover", '<path d="m19 21-7-4-7 4V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2v16z"/>', "#F43F5E", "#F43F5E", 2.0),
    
    # 14: Menu normal (#A1A1AA)
    ("menu_normal", '<line x1="4" x2="20" y1="12" y2="12"/><line x1="4" x2="20" y1="6" y2="6"/><line x1="4" x2="20" y1="18" y2="18"/>', "#A1A1AA", "none", 2.0),
    # 15: Menu hover (#FFFFFF)
    ("menu_hover", '<line x1="4" x2="20" y1="12" y2="12"/><line x1="4" x2="20" y1="6" y2="6"/><line x1="4" x2="20" y1="18" y2="18"/>', "#FFFFFF", "none", 2.0),
    
    # 16: Plus (New tab) normal (#A1A1AA)
    ("plus_normal", '<path d="M5 12h14"/><path d="M12 5v14"/>', "#A1A1AA", "none", 2.0),
    # 17: Plus hover (#E11D48)
    ("plus_hover", '<path d="M5 12h14"/><path d="M12 5v14"/>', "#E11D48", "none", 2.0),
    
    # 18: Close (X) normal (#71717A)
    ("close_normal", '<path d="M18 6 6 18"/><path d="m6 6 12 12"/>', "#71717A", "none", 2.0),
    # 19: Close (X) active tab (#A1A1AA)
    ("close_active", '<path d="M18 6 6 18"/><path d="m6 6 12 12"/>', "#A1A1AA", "none", 2.0),
    # 20: Close (X) hover (#E11D48)
    ("close_hover", '<path d="M18 6 6 18"/><path d="m6 6 12 12"/>', "#E11D48", "none", 2.0),

    # 21: Caption Minimize normal (#A1A1AA)
    ("cap_min_normal", '<line x1="4" x2="20" y1="12" y2="12"/>', "#A1A1AA", "none", 2.0),
    # 22: Caption Minimize hover (#FFFFFF)
    ("cap_min_hover", '<line x1="4" x2="20" y1="12" y2="12"/>', "#FFFFFF", "none", 2.0),
    # 23: Caption Maximize normal (#A1A1AA)
    ("cap_max_normal", '<rect x="4" y="4" width="16" height="16" rx="1"/>', "#A1A1AA", "none", 2.0),
    # 24: Caption Maximize hover (#FFFFFF)
    ("cap_max_hover", '<rect x="4" y="4" width="16" height="16" rx="1"/>', "#FFFFFF", "none", 2.0),
    # 25: Caption Restore normal (#A1A1AA)
    ("cap_restore_normal", '<path d="M8 4h12v12"/><rect x="4" y="8" width="12" height="12" rx="1"/>', "#A1A1AA", "none", 2.0),
    # 26: Caption Restore hover (#FFFFFF)
    ("cap_restore_hover", '<path d="M8 4h12v12"/><rect x="4" y="8" width="12" height="12" rx="1"/>', "#FFFFFF", "none", 2.0),
    # 27: Caption Close normal (#A1A1AA)
    ("cap_close_normal", '<path d="M18 6 6 18"/><path d="m6 6 12 12"/>', "#A1A1AA", "none", 2.0),
    # 28: Caption Close hover (#FFFFFF)
    ("cap_close_hover", '<path d="M18 6 6 18"/><path d="m6 6 12 12"/>', "#FFFFFF", "none", 2.0),
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
    logo_pil_resized = logo_pil.resize((ICON_SIZE - 2, ICON_SIZE - 2), Image.Resampling.LANCZOS)
    logo_rgba = np.array(logo_pil_resized)
    # Convert RGBA to premultiplied ARGB QImage
    logo_qimg = QImage(logo_rgba.data, logo_pil_resized.width, logo_pil_resized.height, QImage.Format.Format_RGBA8888)
    painter.drawImage(1, 1, logo_qimg)

# Draw Lucide icons
for i in range(1, num_icons):
    name, path_data, stroke, fill, stroke_w = lucide_defs[i]
    svg = f'<svg xmlns="http://www.w3.org/2000/svg" width="{ICON_SIZE}" height="{ICON_SIZE}" viewBox="0 0 24 24" fill="{fill}" stroke="{stroke}" stroke-width="{stroke_w}" stroke-linecap="round" stroke-linejoin="round">{path_data}</svg>'
    renderer = QSvgRenderer(QByteArray(svg.encode('utf-8')))
    x_offset = i * ICON_SIZE
    from PyQt6.QtCore import QRectF
    # 3px inset on all sides -> 18x18 icon in 24x24 box for perfect visual optical balance
    renderer.render(painter, QRectF(x_offset + 3.0, 3.0, 18.0, 18.0))

painter.end()

out_png = os.path.abspath('assets/branding/chrome_icons_atlas.png')
atlas_qimg.save(out_png)

# Also dump raw premultiplied BGRA bytes for zero-overhead include_bytes! in Rust
# Format_ARGB32_Premultiplied in memory on little-endian Windows is BGRA!
# Let's extract bytes
ptr = atlas_qimg.bits()
ptr.setsize(atlas_w * atlas_h * 4)
raw_bytes = bytes(ptr)

out_bin = os.path.abspath('assets/branding/chrome_icons_atlas.bin')
with open(out_bin, 'wb') as f:
    f.write(raw_bytes)

print(f"Icon atlas generated! Icons: {num_icons}, Size: {atlas_w}x{atlas_h}, Bytes: {len(raw_bytes)}")
