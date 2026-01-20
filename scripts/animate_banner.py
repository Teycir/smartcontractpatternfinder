from PIL import Image, ImageEnhance, ImageDraw
import os

def create_scanline_animation(input_path, output_path, frames=40):
    try:
        # Load the original image
        original = Image.open(input_path).convert('RGBA')
        width, height = original.size
        
        # Create a list to hold the frames
        animation_frames = []
        
        # Calculate scanline movement
        step = height // frames
        
        for i in range(frames):
            # Create a copy of the original
            frame = original.copy()
            
            # Create a drawing context overlay
            overlay = Image.new('RGBA', (width, height), (0, 0, 0, 0))
            draw = ImageDraw.Draw(overlay)
            
            # Y position of the scanline
            y = (i * step) % height
            
            # Draw the scanline (cyan, glowing)
            # Main line
            draw.line([(0, y), (width, y)], fill=(0, 255, 255, 200), width=2)
            # Glow effect (fading lines above and below)
            for offset in range(1, 5):
                alpha = int(200 / (offset * 2))
                draw.line([(0, y-offset), (width, y-offset)], fill=(0, 255, 255, alpha), width=1)
                draw.line([(0, y+offset), (width, y+offset)], fill=(0, 255, 255, alpha), width=1)
            
            # Composite the overlay onto the frame
            frame = Image.alpha_composite(frame, overlay)
            
            # Convert back to RGB (GIF doesn't support partial transparency well, but we need it for the overlay)
            # Actually, let's keep it simple and convert to RGB
            frame_rgb = frame.convert('RGB')
            
            animation_frames.append(frame_rgb)
            
        print(f"Generating GIF with {len(animation_frames)} frames...")
        
        # Save as GIF
        animation_frames[0].save(
            output_path,
            save_all=True,
            append_images=animation_frames[1:],
            duration=50,   # Faster frames for smooth scanline
            loop=0,
            optimize=False # Optimization often creates artifacts with moving lines
        )
        print(f"Successfully created scanline banner at: {output_path}")
        
    except Exception as e:
        print(f"Error creating animation: {e}")

if __name__ == "__main__":
    base_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    input_img = os.path.join(base_dir, "assets", "scpf_banner.png")
    output_gif = os.path.join(base_dir, "assets", "scpf_banner.gif")
    
    if not os.path.exists(input_img):
        print(f"Error: Input image not found at {input_img}")
    else:
        create_scanline_animation(input_img, output_gif)
