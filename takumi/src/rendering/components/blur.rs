use image::RgbaImage;

use crate::Result;
use crate::rendering::{BufferPool, premultiply_alpha, unpremultiply_alpha};

const PIXEL_STRIDE: usize = 4;

/// Specifies the type of blur operation, which affects how the CSS radius is interpreted.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlurType {
  /// CSS `filter: blur()` - radius equals σ (standard deviation).
  Filter,
  /// CSS `box-shadow` / `text-shadow` blur - radius equals 2σ.
  Shadow,
}

impl BlurType {
  #[inline]
  pub fn to_sigma(self, css_radius: f32) -> f32 {
    match self {
      BlurType::Filter => css_radius,
      BlurType::Shadow => css_radius * 0.5,
    }
  }

  #[inline]
  pub fn extent_multiplier(self) -> f32 {
    match self {
      BlurType::Filter => 3.0,
      BlurType::Shadow => 1.5,
    }
  }
}

#[derive(Clone, Copy)]
struct BlurPassParams {
  width: u32,
  height: u32,
  radius: u32,
  stride: usize,
  mul_val: u32,
  shg: i32,
}

pub(crate) enum BlurFormat<'a> {
  Rgba(&'a mut RgbaImage),
  Alpha {
    data: &'a mut [u8],
    width: u32,
    height: u32,
  },
}

impl<'a> BlurFormat<'a> {
  pub fn width(&self) -> u32 {
    match self {
      Self::Rgba(img) => img.width(),
      Self::Alpha { width, .. } => *width,
    }
  }

  pub fn height(&self) -> u32 {
    match self {
      Self::Rgba(img) => img.height(),
      Self::Alpha { height, .. } => *height,
    }
  }
}

/// Applies a Gaussian approximation using 3-pass Box Blur.
pub(crate) fn apply_blur(
  mut format: BlurFormat<'_>,
  radius: f32,
  blur_type: BlurType,
  pool: &mut BufferPool,
) -> Result<()> {
  let sigma = blur_type.to_sigma(radius);
  if sigma <= 0.5 {
    return Ok(());
  }

  let width = format.width();
  let height = format.height();
  if width == 0 || height == 0 {
    return Ok(());
  }

  let box_radius = (((4.0 * sigma * sigma + 1.0).sqrt() - 1.0) * 0.5)
    .round()
    .max(1.0) as u32;

  let div = 2 * box_radius + 1;
  let (mul_val, shg) = compute_mul_shg(div);

  let stride = match format {
    BlurFormat::Rgba(_) => width as usize * PIXEL_STRIDE,
    BlurFormat::Alpha { .. } => width as usize,
  };

  let pass_params = BlurPassParams {
    width,
    height,
    radius: box_radius,
    stride,
    mul_val,
    shg,
  };

  let mut col_sums = vec![0u32; stride];

  match format {
    BlurFormat::Rgba(ref mut image) => {
      for pixel in image.pixels_mut() {
        premultiply_alpha(pixel);
      }

      let mut temp_image = pool.acquire_image(width, height)?;
      let temp_data = &mut *temp_image;
      let img_data = &mut ***image;

      for _ in 0..3 {
        box_blur_h::<4>(img_data, temp_data, pass_params);
        box_blur_v(temp_data, img_data, pass_params, &mut col_sums);
      }

      pool.release_image(temp_image);

      for pixel in image.pixels_mut() {
        unpremultiply_alpha(pixel);
      }
    }
    BlurFormat::Alpha { ref mut data, .. } => {
      let mut temp_image = pool.acquire((width * height) as usize);
      let temp_data = &mut *temp_image;

      for _ in 0..3 {
        box_blur_h::<1>(data, temp_data, pass_params);
        box_blur_v(temp_data, data, pass_params, &mut col_sums);
      }

      pool.release(temp_image);
    }
  }

  Ok(())
}

/// Horizontal Box Blur Pass
// Kept as a range loop for forced unrolling and to avoid iterator overhead
#[allow(clippy::needless_range_loop)]
fn box_blur_h<const STRIDE: usize>(src: &[u8], dst: &mut [u8], params: BlurPassParams) {
  let r = params.radius as usize;
  let w = params.width as usize;
  let mul = params.mul_val;
  let stride = params.stride;

  assert!(src.len() >= params.height as usize * stride);
  assert!(dst.len() >= params.height as usize * stride);

  for y in 0..params.height as usize {
    let line_offset = y * stride;
    let mut sum = [0u32; STRIDE];

    let first_px = line_offset;
    for c in 0..STRIDE {
      sum[c] = unsafe { *src.get_unchecked(first_px + c) } as u32 * (r as u32 + 1);
    }

    for dx in 1..=r {
      let px = dx.min(w - 1);
      let src_offset = line_offset + px * STRIDE;
      for c in 0..STRIDE {
        sum[c] += unsafe { *src.get_unchecked(src_offset + c) } as u32;
      }
    }

    let left_end = (r + 1).min(w);
    for x in 0..left_end {
      let out_offset = line_offset + x * STRIDE;
      let entering_x = (x + r + 1).min(w - 1);
      let entering_offset = line_offset + entering_x * STRIDE;

      for c in 0..STRIDE {
        unsafe {
          *dst.get_unchecked_mut(out_offset + c) = ((sum[c] * mul) >> params.shg) as u8;
          sum[c] += *src.get_unchecked(entering_offset + c) as u32;
          sum[c] -= *src.get_unchecked(first_px + c) as u32;
        }
      }
    }

    let middle_end = w.saturating_sub(r + 1).max(left_end);
    for x in left_end..middle_end {
      let out_offset = line_offset + x * STRIDE;
      let leaving_offset = line_offset + (x - r) * STRIDE;
      let entering_offset = line_offset + (x + r + 1) * STRIDE;

      for c in 0..STRIDE {
        unsafe {
          *dst.get_unchecked_mut(out_offset + c) = ((sum[c] * mul) >> params.shg) as u8;
          sum[c] += *src.get_unchecked(entering_offset + c) as u32;
          sum[c] -= *src.get_unchecked(leaving_offset + c) as u32;
        }
      }
    }

    let last_px = line_offset + (w - 1) * STRIDE;
    for x in middle_end..w {
      let out_offset = line_offset + x * STRIDE;
      let leaving_offset = line_offset + (x - r) * STRIDE;

      for c in 0..STRIDE {
        unsafe {
          *dst.get_unchecked_mut(out_offset + c) = ((sum[c] * mul) >> params.shg) as u8;
          sum[c] += *src.get_unchecked(last_px + c) as u32;
          sum[c] -= *src.get_unchecked(leaving_offset + c) as u32;
        }
      }
    }
  }
}

/// Vertical Box Blur Pass
// Kept as a range loop for forced unrolling and to avoid iterator overhead in WASM
#[allow(clippy::needless_range_loop)]
fn box_blur_v(src: &[u8], dst: &mut [u8], params: BlurPassParams, sums: &mut [u32]) {
  let r = params.radius as usize;
  let h = params.height as usize;
  let mul = params.mul_val;
  let stride = params.stride;

  assert!(src.len() >= params.height as usize * stride);
  assert!(dst.len() >= params.height as usize * stride);

  // Initialize sums with the first row repeated
  for x in 0..stride {
    sums[x] = unsafe { *src.get_unchecked(x) } as u32 * (r as u32 + 1);
  }

  // Add trailing edge
  for dy in 1..=r {
    let py = dy.min(h - 1);
    let row_offset = py * stride;
    for x in 0..stride {
      sums[x] += unsafe { *src.get_unchecked(row_offset + x) } as u32;
    }
  }

  let left_end = (r + 1).min(h);
  for y in 0..left_end {
    let out_offset = y * stride;
    let entering_y = (y + r + 1).min(h - 1);
    let entering_row = entering_y * stride;

    for x in 0..stride {
      unsafe {
        *dst.get_unchecked_mut(out_offset + x) = ((sums[x] * mul) >> params.shg) as u8;
        sums[x] += *src.get_unchecked(entering_row + x) as u32;
        sums[x] -= *src.get_unchecked(x) as u32;
      }
    }
  }

  let middle_end = h.saturating_sub(r + 1).max(left_end);
  for y in left_end..middle_end {
    let out_offset = y * stride;
    let leaving_row = (y - r) * stride;
    let entering_row = (y + r + 1) * stride;

    for x in 0..stride {
      unsafe {
        *dst.get_unchecked_mut(out_offset + x) = ((sums[x] * mul) >> params.shg) as u8;
        sums[x] += *src.get_unchecked(entering_row + x) as u32;
        sums[x] -= *src.get_unchecked(leaving_row + x) as u32;
      }
    }
  }

  for y in middle_end..h {
    let out_offset = y * stride;
    let leaving_row = (y - r) * stride;
    let last_row = (h - 1) * stride;

    for x in 0..stride {
      unsafe {
        *dst.get_unchecked_mut(out_offset + x) = ((sums[x] * mul) >> params.shg) as u8;
        sums[x] += *src.get_unchecked(last_row + x) as u32;
        sums[x] -= *src.get_unchecked(leaving_row + x) as u32;
      }
    }
  }
}

#[inline(always)]
fn compute_mul_shg(d: u32) -> (u32, i32) {
  let shg = 23;
  let mul = ((1u64 << shg) as f64 / d as f64).round() as u32;
  (mul, shg)
}
