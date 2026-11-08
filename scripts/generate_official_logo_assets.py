import os
import shutil
import base64
from PIL import Image
import numpy as np

src_jpg = r'C:/Users/User/.gemini/antigravity/brain/c9d40ba3-2e1a-46bf-8d38-10499c0b2fbb/.user_uploaded/media_1794120793001_8ff073eb.jpg'
assets_branding = os.path.abspath('assets/branding')
assets_icons = os.path.abspath('assets/icons')

im = Image.open(src_jpg)

# Center of logo content
# Bounding box of content: x=123..872, y=72..634
# cx = 497.5, cy = 353.0
cx = 497.5
cy = 353.0
box_size = 820
half = box_size / 2

# 1. Create square image on black background
sq_black = Image.new('RGB', (box_size, box_size), (0, 0, 0))
crop_x0 = int(cx - half)
crop_x1 = int(cx + half)
crop_y0 = int(cy - half)
crop_y1 = int(cy + half)

src_x0 = max(0, crop_x0)
src_y0 = max(0, crop_y0)
src_x1 = min(im.width, crop_x1)
src_y1 = min(im.height, crop_y1)

crop_im = im.crop((src_x0, src_y0, src_x1, src_y1))
dst_x = src_x0 - crop_x0
dst_y = src_y0 - crop_y0
sq_black.paste(crop_im, (dst_x, dst_y))

# 2. Create square transparent image with clean alpha
arr = np.array(sq_black).astype(np.float32)
max_c = np.maximum.reduce([arr[:,:,0], arr[:,:,1], arr[:,:,2]])

# Alpha ramp: where max_c < 3, alpha = 0.
# Where max_c >= 45, alpha = 255.
# Between 3 and 45, smooth ramp.
alpha = np.clip((max_c - 3.0) / (45.0 - 3.0), 0.0, 1.0) * 255.0

rgba = np.zeros((sq_black.height, sq_black.width, 4), dtype=np.uint8)
a_norm = np.clip(alpha / 255.0, 1e-4, 1.0)

for c in range(3):
    unmult = np.clip(arr[:,:,c] / (a_norm ** 0.8), 0, 255)
    rgba[:,:,c] = unmult.astype(np.uint8)

rgba[:,:,3] = alpha.astype(np.uint8)
sq_trans = Image.fromarray(rgba, 'RGBA')

# Save 512x512
sq_trans_512 = sq_trans.resize((512, 512), Image.Resampling.LANCZOS)
app_icon_path = os.path.join(assets_branding, 'app_icon.png')
sq_trans_512.save(app_icon_path)

# Save 256x256
sq_trans_256 = sq_trans.resize((256, 256), Image.Resampling.LANCZOS)
icon_256_path = os.path.join(assets_branding, 'icon_256.png')
sq_trans_256.save(icon_256_path)

# Save black version 512x512
sq_black_512 = sq_black.resize((512, 512), Image.Resampling.LANCZOS)
app_icon_black_path = os.path.join(assets_branding, 'app_icon_black.png')
sq_black_512.save(app_icon_black_path)

# 3. Generate multi-resolution .ico
ico_sizes = [(256, 256), (128, 128), (64, 64), (48, 48), (32, 32), (24, 24), (16, 16)]
ico_path = os.path.join(assets_branding, 'icon.ico')
sq_trans.save(ico_path, format='ICO', sizes=ico_sizes)

# Copy to assets/icons/nexora.ico
os.makedirs(assets_icons, exist_ok=True)
shutil.copy2(ico_path, os.path.join(assets_icons, 'nexora.ico'))

# 4. Generate base64 data URI
with open(app_icon_path, 'rb') as f:
    b64_str = base64.b64encode(f.read()).decode('utf-8')
    b64_data_uri = f'data:image/png;base64,{b64_str}'

with open(os.path.join(assets_branding, 'app_icon_b64.txt'), 'w', encoding='utf-8') as f:
    f.write(b64_data_uri)

# 5. Generate SVG wrapping the master official logo
svg_content = f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" width="100%" height="100%">
  <image href="{b64_data_uri}" width="512" height="512" />
</svg>'''

with open(os.path.join(assets_branding, 'icon.svg'), 'w', encoding='utf-8') as f:
    f.write(svg_content)

with open(os.path.join(assets_branding, 'logo.svg'), 'w', encoding='utf-8') as f:
    f.write(svg_content)

# 6. Delete old concept files completely
concepts_dir = os.path.join(assets_branding, 'concepts')
if os.path.exists(concepts_dir):
    shutil.rmtree(concepts_dir)
    print(f'Deleted old concepts directory: {concepts_dir}')

dist_concepts = os.path.abspath('dist/Nexora-v0.1.0-portable/assets/branding/concepts')
if os.path.exists(dist_concepts):
    shutil.rmtree(dist_concepts)
    print(f'Deleted old dist concepts directory: {dist_concepts}')

print('All branding assets generated successfully with the official logo!')
