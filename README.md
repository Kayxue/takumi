<div align="center">
  <img src="./assets/images/takumi.svg" alt="Takumi" width="64" />

# Takumi

**Turn JSX into production-ready images fast.**  
OG cards, banners, and lightweight animations from one Rust engine for Node.js and WebAssembly.

[Docs](https://takumi.kane.tw/docs/) · [Playground](https://takumi.kane.tw/playground)

</div>

Takumi is inspired by [satori](https://github.com/vercel/satori), with a stronger focus on portability and performance.

## Why teams pick Takumi

- **One-pass rendering** with no SVG-to-image two-step pipeline.
- **JSX in, pixels out** with cross-runtime delivery.
- **Node native speed** with WASM fallback for edge and browser workers.
- **Production text stack**: variable fonts, COLR, WOFF2, and RTL.
- **Rich output formats**: WebP/PNG/JPEG/APNG or FFmpeg pipelines.

## Performance

In [Image Bench](https://image-bench.kane.tw/), Takumi is typically **2-10x faster** than `next/og`.
See benchmark runs and templates on [Image Bench](https://image-bench.kane.tw/).

## First render in 30 seconds

```bash
npm i @takumi-rs/image-response
```

```tsx
import { ImageResponse } from "@takumi-rs/image-response";

export function GET() {
  return new ImageResponse(
    <div tw="w-full h-full flex items-center justify-center bg-white">
      <h1 tw="text-6xl font-bold">Hello from Takumi</h1>
    </div>,
    {
      width: 1200,
      height: 630,
      format: "webp",
    },
  );
}
```

For runtime-specific setup (Next.js, Vite SSR, Nitro, Cloudflare, Turbopack), see [Docs](https://takumi.kane.tw/docs/).

## What you can build

- Open Graph and social cards
- Blog covers and marketing banners
- JSX-powered dynamic snapshots
- Component-based motion graphics

## Showcase

|                        Takumi OG image [(source)](./example/twitter-images/components/og-image.tsx)                         |                        Package OG card [(source)](./example/twitter-images/components/package-og-image.tsx)                        |
| :-------------------------------------------------------------------------------------------------------------------------: | :--------------------------------------------------------------------------------------------------------------------------------: |
|                 <img src="./example/twitter-images/output/og-image.png" alt="Takumi OG Image" width="100%">                 |                <img src="./example/twitter-images/output/package-og-image.png" alt="Package OG Image" width="100%">                |
|                **Prisma-style API card** [(source)](./example/twitter-images/components/prisma-og-image.tsx)                |                      **X-style social post** [(source)](./example/twitter-images/components/x-post-image.tsx)                      |
|             <img src="./example/twitter-images/output/prisma-og-image.png" alt="Prisma OG Image" width="100%">              |                 <img src="./example/twitter-images/output/x-post-image.png" alt="X-style Post Image" width="100%">                 |
|                    **Keyframe Animation** [(source)](./example/ffmpeg-keyframe-animation/src/index.tsx)                     |                                       **[shiki-image](https://github.com/pi0/shiki-image)**                                        |
| <video src="./example/ffmpeg-keyframe-animation/output/animation.mp4" width="100%" autoPlay loop muted playsInline></video> | <img src="https://raw.githubusercontent.com/pi0/shiki-image/refs/heads/main/test/.snapshot/image.webp" alt="Shiki Image Example" width="100%"> |

- [(Unofficial) Takumi Playground](https://takumi-playground.kapadiya.net/)

> [!NOTE]
> Showcase submissions are welcome via PR to [showcase.ts](./docs/app/data/showcase.ts).

## Contributing

PRs are welcome. See [CONTRIBUTING.md](./CONTRIBUTING.md) for setup, tests, fixtures, and changesets.
By participating, you agree to the [Code of Conduct](./CODE_OF_CONDUCT.md).

## Credits

Takumi builds on excellent OSS:
[taffy](https://github.com/DioxusLabs/taffy),
[image](https://github.com/image-rs/image),
[parley](https://github.com/linebender/parley),
[swash](https://github.com/linebender/swash),
[wuff](https://github.com/nicoburns/wuff),
[resvg](https://github.com/linebender/resvg).

## License

Licensed under either of:

- [Apache License, Version 2.0](./LICENSE-APACHE)
- [MIT license](./LICENSE-MIT)

<br/>
<a href="https://vercel.com/oss">
  <img alt="Vercel OSS Program" src="https://vercel.com/oss/program-badge.svg" />
</a>
