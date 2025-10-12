import { fromJsx } from "@takumi-rs/helpers/jsx";
import DocsTemplateV1 from "@takumi-rs/template/docs-template-v1";
import initWasm, { Renderer } from "@takumi-rs/wasm";
import wasmUrl from "@takumi-rs/wasm/takumi_wasm_bg.wasm?url";
import * as React from "react";
import { transform } from "sucrase";
import * as z from "zod/mini";

let renderer: Renderer | undefined;

const optionsSchema = z.object({
  width: z.int().check(z.positive(), z.minimum(1)),
  height: z.int().check(z.positive(), z.minimum(1)),
  quality: z.optional(
    z.int().check(z.positive(), z.minimum(1), z.maximum(100)),
  ),
  format: z.enum(["png", "jpeg", "webp"]),
});

const exportsSchema = z.object({
  default: z.function(),
  options: optionsSchema,
});

initWasm({ module_or_path: wasmUrl }).then(async () => {
  const font = await fetch("/fonts/Geist.woff2").then((r) => r.arrayBuffer());

  renderer = new Renderer();
  renderer.loadFont(new Uint8Array(font));

  self.postMessage({ type: "ready" });
});

function require(module: string) {
  if (module === "@takumi-rs/template/docs-template-v1") return DocsTemplateV1;
}

function transformCode(code: string) {
  return transform(code, {
    transforms: ["jsx", "typescript", "imports"],
    production: true,
  }).code;
}

function evaluateCodeExports(code: string) {
  const exports = {};

  new Function("exports", "require", "React", transformCode(code))(
    exports,
    require,
    React,
  );

  return exportsSchema.parse(exports);
}

self.onmessage = async (event: MessageEvent) => {
  const { type, code } = event.data;

  if (type === "render" && renderer) {
    try {
      const { default: component, options } = evaluateCodeExports(code);
      const node = await fromJsx(
        React.createElement(component as React.JSXElementConstructor<unknown>),
      );

      const start = performance.now();
      const dataUrl = renderer.renderAsDataUrl(
        node,
        options.width,
        options.height,
        options.format,
        options.quality,
      );
      const duration = performance.now() - start;

      self.postMessage({
        type: "render_complete",
        dataUrl,
        duration,
        node,
        options,
      });
    } catch (error) {
      self.postMessage({
        type: "render_error",
        error: error instanceof Error ? error.message : "Unknown error",
      });
    }
  }
};
