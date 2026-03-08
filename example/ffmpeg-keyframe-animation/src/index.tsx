import { readFile } from "node:fs/promises";
import { join, resolve } from "node:path";
import { Renderer } from "@takumi-rs/core";
import { fromJsx } from "@takumi-rs/helpers/jsx";
import { spawn } from "bun";
import { createHighlighter } from "shiki";

const fps = 30;
const durationSeconds = 4;
const totalFrames = fps * durationSeconds;
const devicePixelRatio = 1.2;
const width = 1200 * devicePixelRatio;
const height = 630 * devicePixelRatio;
const outputPath = resolve(import.meta.dir, "../output/animation.mp4");

const demoCode = `
@keyframes pulse {
  0%, 100% {
    transform: scale(1);
    opacity: 0.7;
  }
  50% {
    transform: scale(1.1);
    opacity: 1;
  }
}
`.trim();

const highlighter = await createHighlighter({
  themes: ["github-dark-default"],
  langs: ["css"],
});

const tokens = highlighter.codeToTokens(demoCode, {
  lang: "css",
  theme: "github-dark-default",
});

let tokenAnimationIndex = 0;

const { node: scene, stylesheets } = await fromJsx(
  <>
    <style>{`
      @keyframes windowReveal {
        0% { transform: translateY(20px) scale(0.95); opacity: 0; }
        100% { transform: translateY(0) scale(1); opacity: 1; }
      }
      @keyframes windowExit {
        0% { transform: translateY(0) scale(1); opacity: 1; }
        100% { transform: translateY(-20px) scale(0.95); opacity: 0; }
      }
      @keyframes textReveal {
        0% { transform: translateY(10px); opacity: 0; }
        100% { transform: translateY(0); opacity: 1; }
      }
    `}</style>

    <main tw="relative flex h-full w-full items-center justify-center overflow-hidden">
      <img
        src="background.jpg"
        tw="absolute inset-0 h-full w-full object-cover"
        alt="Abstract wavy background"
      />
      <div tw="absolute inset-0 bg-black/10" />
      <div
        tw="flex font-mono flex-col items-start overflow-hidden rounded-xl bg-black/40 p-8 ring-1 ring-white/30 shadow-2xl shadow-black/50"
        style={{
          backdropFilter: "blur(24px)",
          fontSize: "26px",
          width: "720px",
          animation:
            "windowReveal 0.6s cubic-bezier(0.34, 1.56, 0.64, 1) both, windowExit 0.4s cubic-bezier(0.36, 0, 0.66, -0.56) 3.5s forwards",
        }}
      >
        <div tw="mb-8 flex gap-2.5">
          <div tw="h-3.5 w-3.5 rounded-full bg-[#ff5f56]" />
          <div tw="h-3.5 w-3.5 rounded-full bg-[#ffbd2e]" />
          <div tw="h-3.5 w-3.5 rounded-full bg-[#27c93f]" />
        </div>
        <div tw="flex flex-col gap-1.5 whitespace-pre-wrap pl-2">
          {tokens.tokens.map((line, i) => (
            <div key={i} tw="flex">
              {line.map((token, j) => {
                const delay = 0.3 + tokenAnimationIndex * 0.025;
                tokenAnimationIndex += 1;

                return (
                  <span
                    key={j}
                    style={{
                      color: token.color,
                      opacity: 0,
                      animation: `textReveal 0.15s ease-out ${delay}s forwards`,
                    }}
                  >
                    {token.content}
                  </span>
                );
              })}
            </div>
          ))}
        </div>
        <img
          src="logo.svg"
          alt="Logo"
          tw="absolute"
          style={{
            width: 64,
            height: 64,
            bottom: 40,
            right: 40,
          }}
        />
      </div>
    </main>
  </>,
);

const ffmpeg = spawn(
  [
    "ffmpeg",
    "-y",
    "-f",
    "rawvideo",
    "-pixel_format",
    "rgba",
    "-video_size",
    `${width}x${height}`,
    "-framerate",
    `${fps}`,
    "-i",
    "pipe:0",
    "-vf",
    "format=yuv420p10le",
    "-c:v",
    "libx265",
    "-crf",
    "16",
    "-preset",
    "medium",
    "-tag:v",
    "hvc1",
    outputPath,
  ],
  { stdin: "pipe", stdout: "ignore", stderr: "ignore" },
);

const renderer = new Renderer();

await renderer.putPersistentImage(
  "logo.svg",
  await readFile(join(import.meta.dir, "../../../docs/public/logo.svg")),
);

await renderer.putPersistentImage(
  "background.jpg",
  await readFile(
    join(
      import.meta.dir,
      "../../../assets/images/martin-martz-W0NRebXbsjM-unsplash.jpg",
    ),
  ),
);

console.log(`Rendering ${totalFrames} frames to ${outputPath}...`);

const framePromises = Array.from({ length: totalFrames }, (_, i) => {
  const timeMs = (i / fps) * 1000;
  return renderer.render(scene, {
    width,
    height,
    devicePixelRatio,
    format: "raw",
    stylesheets,
    timeMs,
  });
});

for (let i = 0; i < totalFrames; i++) {
  const frame = await framePromises[i];
  if (!frame) throw new Error("Frame is undefined");

  ffmpeg.stdin.write(frame);
  if (i % fps === 0)
    console.log(
      `  Progress: ${Math.round((i / totalFrames) * 100)
        .toString()
        .padStart(3)}%`,
    );
}

ffmpeg.stdin.end();
const exitCode = await ffmpeg.exited;

if (exitCode === 0) {
  console.log(`\nSuccess! Video saved to ${outputPath}`);
} else {
  console.error(`ffmpeg failed with exit code ${exitCode}`);
  process.exit(1);
}
