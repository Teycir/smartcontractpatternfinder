from PIL import Image, ImageEnhance
import os
import math

def create_breathing_animation(input_path, output_path, frames=30):
    try:
        # Load the original image
        original = Image.open(input_path)
        
        # Create a list to hold the frames
        animation_frames = []
        
        # Generate frames
        for i in range(frames):
            # Calculate a sine wave factor for 'breathing' effect
            # Varies between 0.8 and 1.2
            factor = 1.0 + 0.2 * math.sin(i * 2 * math.pi / frames)
            
            # Enhance brightness based on the factor
            enhancer = ImageEnhance.Brightness(original)
            enhanced_frame = enhancer.enhance(factor)
            
            animation_frames.append(enhanced_frame)
            
        # Save as GIF
        animation_frames[0].save(
            output_path,
            save_all=True,
            append_images=animation_frames[1:],
            duration=100,  # 100ms per frame = 10fps
            loop=0,        # 0 means loop forever
            optimize=True
        )
        print(f"Successfully created animated banner at: {output_path}")
        
    except Exception as e:
        print(f"Error creating animation: {e}")

if __name__ == "__main__":
    base_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    input_img = os.path.join(base_dir, "assets", "scpf_banner.png")
    output_gif = os.path.join(base_dir, "assets", "scpf_banner.gif")
    
    if not os.path.exists(input_img):
        print(f"Error: Input image not found at {input_img}")
    else:
        create_breathing_animation(input_img, output_gif)
