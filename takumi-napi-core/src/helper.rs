use napi::bindgen_prelude::*;
use napi_derive::napi;
use takumi::{layout::node::Node, resources::task::FetchTaskCollection};

use crate::deserialize_with_tracing;

/// Collects the fetch task urls from the node.
#[napi(ts_args_type = "node: Node")]
pub fn extract_resource_urls(node: Object) -> Result<Vec<String>> {
  let node: Node = deserialize_with_tracing(node)?;

  let mut collection = FetchTaskCollection::default();

  node.collect_fetch_tasks(&mut collection);
  node.collect_style_fetch_tasks(&mut collection);

  Ok(
    collection
      .into_inner()
      .iter()
      .map(|task| task.to_string())
      .collect(),
  )
}
