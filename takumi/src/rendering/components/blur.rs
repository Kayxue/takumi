use image::RgbaImage;
use libblur::{BlurImage, BlurImageMut, CLTParameters, FastBlurChannels, ThreadingPolicy};

/// Specifies the type of blur operation, which affects how the CSS radius is interpreted.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlurType {
  /// CSS `filter: blur()` - radius equals σ (standard deviation).
  /// Visual extent is approximately ±3σ = ±3r.
  Filter,
  /// CSS `box-shadow` / `text-shadow` blur - radius equals 2σ.
  /// Visual extent is approximately ±3σ = ±1.5r.
  Shadow,
}

impl BlurType {
  /// Converts CSS blur radius to the standard deviation (σ).
  #[inline]
  pub fn to_sigma(self, css_radius: f32) -> f32 {
    match self {
      // filter: blur(r) where r = σ
      BlurType::Filter => css_radius,
      // shadow blur(r) where r = 2σ → σ = r/2
      BlurType::Shadow => css_radius * 0.5,
    }
  }

  /// Returns the blur extent multiplier (how far the blur visually extends).
  /// This is used to calculate padding needed for the blur image.
  /// Based on Gaussian blur theory: visible extent ≈ 3σ.
  #[inline]
  pub fn extent_multiplier(self) -> f32 {
    match self {
      // filter: blur(r) where r = σ → extent = 3σ = 3r
      BlurType::Filter => 3.0,
      // shadow blur(r) where r = 2σ → σ = r/2 → extent = 3σ = 1.5r
      BlurType::Shadow => 1.5,
    }
  }
}

/// Applies a blur to an image using libblur's gaussian_box_blur (3-pass box blur).
pub(crate) fn apply_blur(image: &mut RgbaImage, radius: f32, blur_type: BlurType) {
  let sigma = blur_type.to_sigma(radius);
  if sigma <= 0.5 {
    return;
  }

  let (width, height) = image.dimensions();
  if width == 0 || height == 0 {
    return;
  }

  // Convert to premultiplied alpha for correct blur behavior
  premultiply_alpha(image);

  // Create source image reference
  let src = BlurImage::borrow(image.as_ref(), width, height, FastBlurChannels::Channels4);

  // Create destination buffer
  let mut dst_bytes = vec![0u8; (width * height * 4) as usize];
  let mut dst = BlurImageMut::borrow(&mut dst_bytes, width, height, FastBlurChannels::Channels4);

  // Apply gaussian box blur (3-pass box blur approximating Gaussian)
  let params = CLTParameters::new(sigma);
  libblur::gaussian_box_blur(
    &src,
    &mut dst,
    params,
    #[cfg(target_arch = "wasm32")]
    ThreadingPolicy::Single,
    #[cfg(not(target_arch = "wasm32"))]
    ThreadingPolicy::Adaptive,
  )
  .ok();

  // Copy result back to image
  image.as_mut().copy_from_slice(&dst_bytes);

  // Convert back to straight alpha
  unpremultiply_alpha(image);
}

/// Converts an image from straight alpha to premultiplied alpha.
fn premultiply_alpha(image: &mut RgbaImage) {
  for pixel in image.pixels_mut() {
    let a = pixel.0[3] as u16;
    if a == 0 {
      pixel.0[0] = 0;
      pixel.0[1] = 0;
      pixel.0[2] = 0;
    } else if a < 255 {
      pixel.0[0] = fast_div_255(pixel.0[0] as u16 * a);
      pixel.0[1] = fast_div_255(pixel.0[1] as u16 * a);
      pixel.0[2] = fast_div_255(pixel.0[2] as u16 * a);
    }
  }
}

/// Converts an image from premultiplied alpha to straight alpha.
fn unpremultiply_alpha(image: &mut RgbaImage) {
  for pixel in image.pixels_mut() {
    let a = pixel.0[3] as u16;
    if a == 0 {
      pixel.0[0] = 0;
      pixel.0[1] = 0;
      pixel.0[2] = 0;
    } else if a < 255 {
      pixel.0[0] = ((pixel.0[0] as u16 * 255 + a / 2) / a).min(255) as u8;
      pixel.0[1] = ((pixel.0[1] as u16 * 255 + a / 2) / a).min(255) as u8;
      pixel.0[2] = ((pixel.0[2] as u16 * 255 + a / 2) / a).min(255) as u8;
    }
  }
}

/// Fast approximation of integer division by 255.
#[inline(always)]
pub(crate) fn fast_div_255(v: u16) -> u8 {
  ((v + 128 + (v >> 8)) >> 8) as u8
}
