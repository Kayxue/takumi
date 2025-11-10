mod load_font_task;
mod put_persistent_image_task;
mod render_animation_task;
mod render_task;
mod renderer;

use napi::{
  JsString,
  bindgen_prelude::{BufferSlice, Function, Object, PromiseRaw},
};
pub use renderer::Renderer;
use serde::{Deserialize, Deserializer};
use takumi::parley::FontStyle;

#[derive(Deserialize, Default)]
pub(crate) struct FontInput {
  pub name: Option<String>,
  pub weight: Option<f64>,
  pub style: Option<FontStyleInput>,
}

#[derive(Clone, Copy)]
pub struct FontStyleInput(pub FontStyle);

impl<'de> Deserialize<'de> for FontStyleInput {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    Ok(FontStyleInput(FontStyle::parse(&s).unwrap_or_default()))
  }
}

// fetch(url: string): Promise<Response>
pub(crate) type FetchFn<'env> = Function<'env, JsString<'env>, PromiseRaw<'env, Object<'env>>>;

/// arrayBuffer(this: Response): Promise<ArrayBuffer>
pub(crate) type ArrayBufferFn<'env> = Function<'env, (), PromiseRaw<'env, BufferSlice<'env>>>;

pub(crate) enum MaybeInitialized<B, A> {
  Uninitialized(B),
  Initialized(A),
}
