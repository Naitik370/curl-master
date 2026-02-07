from PIL import Image, ImageDraw
import os

# Create icons directory if it doesn't exist
icons_dir = os.path.join('src-tauri', 'icons')
os.makedirs(icons_dir, exist_ok=True)

def create_icon(size, filename):
    img = Image.new('RGBA', (size, size), (102, 126, 234, 255))
    draw = ImageDraw.Draw(img)
    border_width = max(2, size // 64)
    padding = size // 8
    draw.rectangle([padding, padding, size-padding, size-padding], 
                   outline=(255, 255, 255, 255), width=border_width)
    
    output_path = os.path.join(icons_dir, filename)
    img.save(output_path, format='PNG')
    print(f"Created {output_path}")

# Create various sizes
create_icon(32, '32x32.png')
create_icon(128, '128x128.png')
create_icon(256, '128x128@2x.png')
create_icon(512, 'icon.png')

print("All icons created successfully!")
