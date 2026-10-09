#!/usr/bin/env python3
import os
import math
from PIL import Image, ImageDraw, ImageFont, ImageFilter

OUTPUT_DIR = "/home/chomiam/Projets/noos_htpc/themes/grub-theme"
ICONS_DIR = os.path.join(OUTPUT_DIR, "icons")
os.makedirs(ICONS_DIR, exist_ok=True)

WIDTH, HEIGHT = 1920, 1080

# Catppuccin Mocha Palette
CRUST = (17, 17, 27)         # #11111b
MANTLE = (24, 24, 37)        # #181825
BASE = (30, 30, 46)          # #1e1e2e
SURFACE0 = (49, 50, 68)      # #313244
SURFACE1 = (69, 71, 90)      # #45475a
SURFACE2 = (88, 91, 112)     # #585b70
OVERLAY0 = (108, 112, 134)   # #6c7086
SUBTEXT0 = (166, 173, 200)   # #a6adc8
TEXT = (205, 214, 244)       # #cdd6f4
BLUE = (137, 180, 250)       # #89b4fa
SAPPHIRE = (116, 199, 236)   # #74c7ec
MAUVE = (203, 166, 247)      # #cba6f7
GREEN = (166, 227, 161)      # #a6e3a1

print("1. Generating background.png (1920x1080)...")
# Base image with subtle radial gradient from center mantle to outer crust
bg = Image.new("RGBA", (WIDTH, HEIGHT), CRUST + (255,))
draw = ImageDraw.Draw(bg)

cx, cy = WIDTH // 2, HEIGHT // 2
max_radius = math.hypot(cx, cy)

for y in range(HEIGHT):
    # Vertical gradient + vignette
    ratio_y = (y / HEIGHT)
    r = int(MANTLE[0] + (CRUST[0] - MANTLE[0]) * abs(ratio_y - 0.45) * 1.5)
    g = int(MANTLE[1] + (CRUST[1] - MANTLE[1]) * abs(ratio_y - 0.45) * 1.5)
    b = int(MANTLE[2] + (CRUST[2] - MANTLE[2]) * abs(ratio_y - 0.45) * 1.5)
    r = max(CRUST[0], min(BASE[0], r))
    g = max(CRUST[1], min(BASE[1], g))
    b = max(CRUST[2], min(BASE[2], b))
    draw.line([(0, y), (WIDTH, y)], fill=(r, g, b, 255))

# Soft ambient glow behind logo and menu card
glow = Image.new("RGBA", (WIDTH, HEIGHT), (0, 0, 0, 0))
glow_draw = ImageDraw.Draw(glow)
glow_draw.ellipse([cx - 450, 100, cx + 450, 600], fill=(BLUE[0], BLUE[1], BLUE[2], 18))
glow_draw.ellipse([cx - 300, 300, cx + 300, 800], fill=(MAUVE[0], MAUVE[1], MAUVE[2], 12))
glow = glow.filter(ImageFilter.GaussianBlur(80))
bg = Image.alpha_composite(bg, glow)

# Central Glass Card Container for the GRUB boot menu
card_w, card_h = 900, 520
card_x = (WIDTH - card_w) // 2
card_y = 260
corner_r = 20

# Shadow for the card
shadow = Image.new("RGBA", (WIDTH, HEIGHT), (0, 0, 0, 0))
shadow_draw = ImageDraw.Draw(shadow)
shadow_draw.rounded_rectangle(
    [card_x - 10, card_y - 5, card_x + card_w + 10, card_y + card_h + 15],
    radius=corner_r + 4,
    fill=(0, 0, 0, 120)
)
shadow = shadow.filter(ImageFilter.GaussianBlur(25))
bg = Image.alpha_composite(bg, shadow)

# Card body (glassy dark)
card = Image.new("RGBA", (WIDTH, HEIGHT), (0, 0, 0, 0))
card_draw = ImageDraw.Draw(card)
card_draw.rounded_rectangle(
    [card_x, card_y, card_x + card_w, card_y + card_h],
    radius=corner_r,
    fill=(BASE[0], BASE[1], BASE[2], 210),
    outline=(SURFACE1[0], SURFACE1[1], SURFACE1[2], 200),
    width=2
)
# Top accent border on card
card_draw.arc(
    [card_x, card_y, card_x + corner_r * 2, card_y + corner_r * 2],
    180, 270, fill=BLUE + (255,), width=2
)
card_draw.line(
    [(card_x + corner_r, card_y), (card_x + card_w - corner_r, card_y)],
    fill=BLUE + (255,), width=2
)
card_draw.arc(
    [card_x + card_w - corner_r * 2, card_y, card_x + card_w, card_y + corner_r * 2],
    270, 360, fill=BLUE + (255,), width=2
)

bg = Image.alpha_composite(bg, card)

# Load and place Noos logo
logo_path = "/home/chomiam/Projets/noos_htpc/logo.png"
if os.path.exists(logo_path):
    logo = Image.open(logo_path).convert("RGBA")
    # Scale logo nicely
    target_logo_w = 340
    ratio = target_logo_w / logo.width
    target_logo_h = int(logo.height * ratio)
    logo_resized = logo.resize((target_logo_w, target_logo_h), Image.Resampling.LANCZOS)
    
    logo_x = (WIDTH - target_logo_w) // 2
    logo_y = 90
    
    # Logo drop shadow
    logo_shadow = Image.new("RGBA", (WIDTH, HEIGHT), (0, 0, 0, 0))
    logo_shadow.paste(logo_resized, (logo_x, logo_y + 4), logo_resized)
    # recolor shadow to black
    r, g, b, a = logo_shadow.split()
    black_shadow = Image.merge("RGBA", (Image.new("L", r.size, 0), Image.new("L", g.size, 0), Image.new("L", b.size, 0), a))
    black_shadow = black_shadow.filter(ImageFilter.GaussianBlur(8))
    bg = Image.alpha_composite(bg, black_shadow)
    
    # Paste actual logo
    bg.paste(logo_resized, (logo_x, logo_y), logo_resized)

# Footer shortcuts helper text
footer_draw = ImageDraw.Draw(bg)
try:
    font_footer = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 15)
    font_badge = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", 13)
except Exception:
    font_footer = ImageFont.load_default()
    font_badge = font_footer

shortcuts_text = "Entrée: Démarrer    •    ↑/↓: Naviguer    •    E: Options    •    C: Console"
footer_bbox = footer_draw.textbbox((0, 0), shortcuts_text, font=font_footer)
footer_w = footer_bbox[2] - footer_bbox[0]
footer_draw.text(((WIDTH - footer_w) // 2, 810), shortcuts_text, fill=OVERLAY0 + (255,), font=font_footer)

watermark = "Noos HTPC • Système Multimédia Déclaratif"
wm_bbox = footer_draw.textbbox((0, 0), watermark, font=font_badge)
wm_w = wm_bbox[2] - wm_bbox[0]
footer_draw.text(((WIDTH - wm_w) // 2, 1030), watermark, fill=SURFACE1 + (255,), font=font_badge)

bg.convert("RGB").save(os.path.join(OUTPUT_DIR, "background.png"), "PNG")
print("-> background.png generated.")

# 2. Generating selection pixmaps (select_w.png, select_c.png, select_e.png)
print("2. Generating selection pixmaps...")
sel_h = 48
sel_corner_w = 12

# select_w (left round end)
img_w = Image.new("RGBA", (sel_corner_w, sel_h), (0, 0, 0, 0))
draw_w = ImageDraw.Draw(img_w)
draw_w.rounded_rectangle([0, 0, sel_corner_w * 2, sel_h - 1], radius=8, fill=SURFACE0 + (230,), outline=BLUE + (255,), width=1)
img_w.save(os.path.join(OUTPUT_DIR, "select_w.png"), "PNG")

# select_c (center 1px wide stretched)
img_c = Image.new("RGBA", (1, sel_h), (0, 0, 0, 0))
draw_c = ImageDraw.Draw(img_c)
draw_c.line([(0, 0), (0, sel_h - 1)], fill=SURFACE0 + (230,), width=1)
draw_c.point([(0, 0), (0, sel_h - 1)], fill=BLUE + (255,))
img_c.save(os.path.join(OUTPUT_DIR, "select_c.png"), "PNG")

# select_e (right round end)
img_e = Image.new("RGBA", (sel_corner_w, sel_h), (0, 0, 0, 0))
draw_e = ImageDraw.Draw(img_e)
draw_e.rounded_rectangle([-sel_corner_w, 0, sel_corner_w - 1, sel_h - 1], radius=8, fill=SURFACE0 + (230,), outline=BLUE + (255,), width=1)
img_e.save(os.path.join(OUTPUT_DIR, "select_e.png"), "PNG")

# 3. Generating progress bar pixmaps
print("3. Generating progress bar pixmaps...")
pb_h = 10
pb_corner = 5

# Progress bar frame (empty background)
pb_frame_w = Image.new("RGBA", (pb_corner, pb_h), (0, 0, 0, 0))
draw_pfw = ImageDraw.Draw(pb_frame_w)
draw_pfw.rounded_rectangle([0, 0, pb_corner * 2, pb_h - 1], radius=4, fill=MANTLE + (255,), outline=SURFACE1 + (255,), width=1)
pb_frame_w.save(os.path.join(OUTPUT_DIR, "progress_bar_w.png"), "PNG")

pb_frame_c = Image.new("RGBA", (1, pb_h), (0, 0, 0, 0))
draw_pfc = ImageDraw.Draw(pb_frame_c)
draw_pfc.line([(0, 0), (0, pb_h - 1)], fill=MANTLE + (255,), width=1)
draw_pfc.point([(0, 0), (0, pb_h - 1)], fill=SURFACE1 + (255,))
pb_frame_c.save(os.path.join(OUTPUT_DIR, "progress_bar_c.png"), "PNG")

pb_frame_e = Image.new("RGBA", (pb_corner, pb_h), (0, 0, 0, 0))
draw_pfe = ImageDraw.Draw(pb_frame_e)
draw_pfe.rounded_rectangle([-pb_corner, 0, pb_corner - 1, pb_h - 1], radius=4, fill=MANTLE + (255,), outline=SURFACE1 + (255,), width=1)
pb_frame_e.save(os.path.join(OUTPUT_DIR, "progress_bar_e.png"), "PNG")

# Progress highlight (filled bar)
pb_hl_w = Image.new("RGBA", (pb_corner, pb_h), (0, 0, 0, 0))
draw_phw = ImageDraw.Draw(pb_hl_w)
draw_phw.rounded_rectangle([0, 0, pb_corner * 2, pb_h - 1], radius=4, fill=BLUE + (255,), outline=SAPPHIRE + (255,), width=1)
pb_hl_w.save(os.path.join(OUTPUT_DIR, "progress_highlight_w.png"), "PNG")

pb_hl_c = Image.new("RGBA", (1, pb_h), (0, 0, 0, 0))
draw_phc = ImageDraw.Draw(pb_hl_c)
draw_phc.line([(0, 0), (0, pb_h - 1)], fill=BLUE + (255,), width=1)
draw_phc.point([(0, 0), (0, pb_h - 1)], fill=SAPPHIRE + (255,))
pb_hl_c.save(os.path.join(OUTPUT_DIR, "progress_highlight_c.png"), "PNG")

pb_hl_e = Image.new("RGBA", (pb_corner, pb_h), (0, 0, 0, 0))
draw_phe = ImageDraw.Draw(pb_hl_e)
draw_phe.rounded_rectangle([-pb_corner, 0, pb_corner - 1, pb_h - 1], radius=4, fill=BLUE + (255,), outline=SAPPHIRE + (255,), width=1)
pb_hl_e.save(os.path.join(OUTPUT_DIR, "progress_highlight_e.png"), "PNG")

# 4. Generating 32x32 clean icons
print("4. Generating theme icons...")

def create_icon(name, draw_fn):
    icon = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(icon)
    draw_fn(d)
    icon.save(os.path.join(ICONS_DIR, f"{name}.png"), "PNG")

# Noos / NixOS primary icon (Modern HTPC / TV console icon)
def draw_noos(d):
    # Rounded TV / Display
    d.rounded_rectangle([3, 4, 28, 22], radius=4, fill=SURFACE0 + (255,), outline=BLUE + (255,), width=2)
    # Screen inner glow
    d.rectangle([6, 7, 25, 19], fill=(BLUE[0], BLUE[1], BLUE[2], 120))
    # Stand
    d.line([(16, 23), (16, 27)], fill=BLUE + (255,), width=2)
    d.line([(11, 27), (21, 27)], fill=BLUE + (255,), width=2)

create_icon("noos-htpc", draw_noos)
create_icon("noos", draw_noos)
create_icon("nixos", draw_noos)
create_icon("gnu-linux", draw_noos)

# Submenu / Rollback icon (Folder / History)
def draw_history(d):
    # Clock / history rollback circle
    d.ellipse([4, 4, 27, 27], outline=MAUVE + (255,), width=2)
    d.line([(16, 16), (16, 10)], fill=MAUVE + (255,), width=2)
    d.line([(16, 16), (21, 16)], fill=MAUVE + (255,), width=2)
    # Arrow head
    d.polygon([(16, 7), (13, 11), (19, 11)], fill=MAUVE + (255,))

create_icon("submenu", draw_history)
create_icon("rollback", draw_history)

# UEFI / Firmware Settings icon (Chip)
def draw_uefi(d):
    d.rectangle([8, 8, 23, 23], fill=SURFACE0 + (255,), outline=GREEN + (255,), width=2)
    for p in [11, 15, 20]:
        d.line([(4, p), (8, p)], fill=GREEN + (255,), width=1)
        d.line([(23, p), (27, p)], fill=GREEN + (255,), width=1)
        d.line([(p, 4), (p, 8)], fill=GREEN + (255,), width=1)
        d.line([(p, 23), (p, 27)], fill=GREEN + (255,), width=1)

create_icon("uefi", draw_uefi)
create_icon("firmware", draw_uefi)

# Reboot icon
def draw_reboot(d):
    d.arc([5, 5, 26, 26], start=30, end=300, fill=SAPPHIRE + (255,), width=2)
    d.polygon([(24, 7), (27, 13), (21, 13)], fill=SAPPHIRE + (255,))

create_icon("reboot", draw_reboot)
create_icon("restart", draw_reboot)

# Shutdown icon
def draw_shutdown(d):
    d.arc([5, 8, 26, 28], start=45, end=315, fill=(243, 139, 168, 255), width=2)
    d.line([(16, 3), (16, 14)], fill=(243, 139, 168, 255), width=2)

create_icon("shutdown", draw_shutdown)

print("-> Icons created.")
