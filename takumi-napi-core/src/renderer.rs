use lru::LruCache;
use napi::{De, bindgen_prelude::*};
use napi_derive::napi;
use serde::de::DeserializeOwned;
use takumi::{
  GlobalContext,
  layout::{Viewport, node::NodeKind},
  parley::{FontWeight, GenericFamily, fontique::FontInfoOverride},
  rendering::ImageOutputFormat,
  resources::{
    image::{ImageSource, load_image_source_from_bytes},
    task::FetchTask,
  },
};

use crate::{
  FetchFn, FontInput, FontInputOwned, load_font_task::LoadFontTask,
  put_persistent_image_task::PutPersistentImageTask, render_animation_task::RenderAnimationTask,
  render_task::RenderTask,
};
use std::{
  num::NonZeroUsize,
  sync::{Arc, Mutex},
};

pub(crate) type ResourceCache = Option<Arc<Mutex<LruCache<FetchTask, Arc<ImageSource>>>>>;

#[napi]
pub struct Renderer {
  global: Arc<GlobalContext>,
  resources_cache: ResourceCache,
}

#[napi(object)]
pub struct RenderOptions<'env> {
  pub width: u32,
  pub height: u32,
  pub format: Option<OutputFormat>,
  pub quality: Option<u8>,
  pub draw_debug_border: Option<bool>,
  pub fetch: Option<FetchFn<'env>>,
}

#[napi(object)]
pub struct AnimationFrameSource<'ctx> {
  #[napi(ts_type = "AnyNode")]
  pub node: Object<'ctx>,
  pub duration_ms: u32,
}

#[napi(object)]
pub struct RenderAnimationOptions {
  pub draw_debug_border: Option<bool>,
  pub width: u32,
  pub height: u32,
  pub format: Option<AnimationOutputFormat>,
}

#[napi(string_enum)]
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AnimationOutputFormat {
  webp,
  apng,
}

#[napi(string_enum)]
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
  webp,
  avif,
  png,
  jpeg,
  WebP,
  Avif,
  Jpeg,
  Png,
  raw,
}

impl From<OutputFormat> for ImageOutputFormat {
  fn from(format: OutputFormat) -> Self {
    match format {
      OutputFormat::WebP => ImageOutputFormat::WebP,
      OutputFormat::Avif => ImageOutputFormat::Avif,
      OutputFormat::Jpeg => ImageOutputFormat::Jpeg,
      OutputFormat::Png => ImageOutputFormat::Png,
      OutputFormat::png => ImageOutputFormat::Png,
      OutputFormat::jpeg => ImageOutputFormat::Jpeg,
      OutputFormat::webp => ImageOutputFormat::WebP,
      OutputFormat::avif => ImageOutputFormat::Avif,
      // SAFETY: It's handled in the render task
      OutputFormat::raw => unreachable!(),
    }
  }
}

#[napi(object)]
pub struct PersistentImage<'ctx> {
  pub src: String,
  #[napi(ts_type = "Buffer | ArrayBuffer")]
  pub data: BufferSlice<'ctx>,
}

#[napi(object)]
#[derive(Default)]
pub struct ConstructRendererOptions<'ctx> {
  pub persistent_images: Option<Vec<PersistentImage<'ctx>>>,
  #[napi(ts_type = "Font[] | undefined")]
  pub fonts: Option<Vec<Object<'ctx>>>,
  pub load_default_fonts: Option<bool>,
  pub resource_cache_capacity: Option<u32>,
}

const EMBEDDED_FONTS: &[(&[u8], &str, GenericFamily)] = &[
  (
    include_bytes!("../../assets/fonts/geist/Geist[wght].woff2"),
    "Geist",
    GenericFamily::SansSerif,
  ),
  (
    include_bytes!("../../assets/fonts/geist/GeistMono[wght].woff2"),
    "Geist Mono",
    GenericFamily::Monospace,
  ),
];

const DEFAULT_RESOURCE_CACHE_CAPACITY: u32 = 8;

#[napi]
impl Renderer {
  #[napi(constructor)]
  pub fn new(env: Env, options: Option<ConstructRendererOptions>) -> Self {
    let options = options.unwrap_or_default();

    let load_default_fonts = options
      .load_default_fonts
      .unwrap_or_else(|| options.fonts.is_none());

    let resource_cache_capacity = options
      .resource_cache_capacity
      .unwrap_or(DEFAULT_RESOURCE_CACHE_CAPACITY);

    let renderer = Self {
      global: Arc::new(GlobalContext::default()),
      resources_cache: if resource_cache_capacity > 0 {
        Some(Arc::new(Mutex::new(LruCache::new(
          NonZeroUsize::new(resource_cache_capacity as usize).unwrap(),
        ))))
      } else {
        None
      },
    };

    if load_default_fonts {
      for (font, name, generic) in EMBEDDED_FONTS {
        renderer
          .global
          .font_context
          .load_and_store(
            font,
            Some(FontInfoOverride {
              family_name: Some(name),
              ..Default::default()
            }),
            Some(*generic),
          )
          .unwrap();
      }
    }

    if let Some(images) = options.persistent_images {
      for image in images {
        let image_source = load_image_source_from_bytes(&image.data).unwrap();

        renderer
          .global
          .persistent_image_store
          .insert(&image.src, image_source);
      }
    }

    if let Some(fonts) = options.fonts {
      for font in fonts {
        if font.is_arraybuffer().unwrap() || font.is_buffer().unwrap() {
          // SAFETY: We know the font is a buffer
          let buffer = unsafe { BufferSlice::from_napi_value(env.raw(), font.raw()).unwrap() };

          renderer
            .global
            .font_context
            .load_and_store(&buffer, None, None)
            .unwrap();

          continue;
        }

        let font: FontInput = unsafe { FontInput::from_napi_value(env.raw(), font.raw()).unwrap() };

        let font_override = FontInfoOverride {
          family_name: font.name.as_deref(),
          style: font.style.map(Into::into),
          weight: font.weight.map(|weight| FontWeight::new(weight as f32)),
          axes: None,
          width: None,
        };

        renderer
          .global
          .font_context
          .load_and_store(&font.data, Some(font_override), None)
          .unwrap();
      }
    }

    renderer
  }

  #[napi]
  pub fn purge_resources_cache(&self) {
    if let Some(resource_cache) = self.resources_cache.as_ref() {
      let mut lock = resource_cache.lock().unwrap();

      lock.clear();
    }
  }

  #[napi]
  pub fn purge_font_cache(&self) {
    self.global.font_context.purge_cache();
  }

  /// @deprecated Use `putPersistentImage` instead (to align with the naming convention for sync/async functions).
  #[napi(
    ts_args_type = "src: string, data: Buffer | ArrayBuffer, signal?: AbortSignal",
    ts_return_type = "Promise<void>"
  )]
  pub fn put_persistent_image_async(
    &self,
    src: String,
    data: Buffer,
    signal: Option<AbortSignal>,
  ) -> AsyncTask<PutPersistentImageTask> {
    self.put_persistent_image(src, data, signal)
  }

  #[napi(
    ts_args_type = "src: string, data: Buffer | ArrayBuffer, signal?: AbortSignal",
    ts_return_type = "Promise<void>"
  )]
  pub fn put_persistent_image(
    &self,
    src: String,
    data: Buffer,
    signal: Option<AbortSignal>,
  ) -> AsyncTask<PutPersistentImageTask> {
    AsyncTask::with_optional_signal(
      PutPersistentImageTask {
        src: Some(src),
        context: Arc::clone(&self.global),
        buffer: data,
      },
      signal,
    )
  }

  /// @deprecated Use `loadFont` instead (to align with the naming convention for sync/async functions).
  #[napi(
    ts_args_type = "data: Font, signal?: AbortSignal",
    ts_return_type = "Promise<number>"
  )]
  pub fn load_font_async(
    &self,
    env: Env,
    data: Object,
    signal: Option<AbortSignal>,
  ) -> AsyncTask<LoadFontTask> {
    self.load_fonts_async(env, vec![data], signal)
  }

  #[napi(
    ts_args_type = "data: Font, signal?: AbortSignal",
    ts_return_type = "Promise<number>"
  )]
  pub fn load_font(
    &self,
    env: Env,
    data: Object,
    signal: Option<AbortSignal>,
  ) -> AsyncTask<LoadFontTask> {
    self.load_fonts_async(env, vec![data], signal)
  }

  /// @deprecated Use `loadFonts` instead (to align with the naming convention for sync/async functions).
  #[napi(
    ts_args_type = "fonts: Font[], signal?: AbortSignal",
    ts_return_type = "Promise<number>"
  )]
  pub fn load_fonts_async(
    &self,
    env: Env,
    fonts: Vec<Object>,
    signal: Option<AbortSignal>,
  ) -> AsyncTask<LoadFontTask> {
    self.load_fonts(env, fonts, signal)
  }

  #[napi(
    ts_args_type = "fonts: Font[], signal?: AbortSignal",
    ts_return_type = "Promise<number>"
  )]
  pub fn load_fonts(
    &self,
    env: Env,
    fonts: Vec<Object>,
    signal: Option<AbortSignal>,
  ) -> AsyncTask<LoadFontTask> {
    let fonts = fonts
      .into_iter()
      .map(|font| {
        if font.is_arraybuffer().unwrap() || font.is_buffer().unwrap() {
          FontInputOwned {
            name: None,
            // SAFETY: We know the font is a buffer
            data: unsafe { Buffer::from_napi_value(env.raw(), font.raw()).unwrap() },
            weight: None,
            style: None,
          }
        } else {
          unsafe { FontInputOwned::from_napi_value(env.raw(), font.raw()).unwrap() }
        }
      })
      .collect();

    AsyncTask::with_optional_signal(
      LoadFontTask {
        context: Arc::clone(&self.global),
        buffers: fonts,
      },
      signal,
    )
  }

  #[napi]
  pub fn clear_image_store(&self) {
    self.global.persistent_image_store.clear();
  }

  #[napi(
    ts_args_type = "source: AnyNode, options: RenderOptions, signal?: AbortSignal",
    ts_return_type = "Promise<Buffer>"
  )]
  pub fn render(
    &self,
    env: Env,
    source: Object,
    options: RenderOptions,
    signal: Option<AbortSignal>,
  ) -> Result<AsyncTask<RenderTask>> {
    let node: NodeKind = deserialize_with_tracing(source)?;

    Ok(AsyncTask::with_optional_signal(
      RenderTask::from_options(
        env,
        node,
        options,
        &self.resources_cache,
        self.global.clone(),
      )?,
      signal,
    ))
  }

  /// @deprecated Use `render` instead (to align with the naming convention for sync/async functions).
  #[napi(
    ts_args_type = "source: AnyNode, options: RenderOptions, signal?: AbortSignal",
    ts_return_type = "Promise<Buffer>"
  )]
  pub fn render_async(
    &mut self,
    env: Env,
    source: Object,
    options: RenderOptions,
    signal: Option<AbortSignal>,
  ) -> Result<AsyncTask<RenderTask>> {
    self.render(env, source, options, signal)
  }

  #[napi(
    ts_args_type = "source: AnimationFrameSource[], options: RenderAnimationOptions, signal?: AbortSignal",
    ts_return_type = "Promise<Buffer>"
  )]
  pub fn render_animation(
    &self,
    source: Vec<AnimationFrameSource>,
    options: RenderAnimationOptions,
    signal: Option<AbortSignal>,
  ) -> Result<AsyncTask<RenderAnimationTask>> {
    let nodes = source
      .into_iter()
      .map(|frame| {
        (
          deserialize_with_tracing(frame.node).unwrap(),
          frame.duration_ms,
        )
      })
      .collect::<Vec<_>>();

    Ok(AsyncTask::with_optional_signal(
      RenderAnimationTask {
        nodes: Some(nodes),
        context: Arc::clone(&self.global),
        viewport: Viewport::new(options.width, options.height),
        format: options.format.unwrap_or(AnimationOutputFormat::webp),
        draw_debug_border: options.draw_debug_border.unwrap_or_default(),
      },
      signal,
    ))
  }
}

fn deserialize_with_tracing<T: DeserializeOwned>(value: Object) -> Result<T> {
  let mut de = De::new(&value);
  T::deserialize(&mut de).map_err(|e| Error::from_reason(e.to_string()))
}
