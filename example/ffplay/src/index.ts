import { type AnyNode, Renderer } from "@takumi-rs/core";
import { container, text } from "@takumi-rs/helpers";
import { spawn } from "bun";

const fps = 60;
const width = 960;
const height = 540;

const renderer = new Renderer();

// Start ffplay with proper flags for raw RGBA input
const ffplay = spawn(
  [
    "ffplay",
    "-f",
    "rawvideo",
    "-pixel_format",
    "rgba",
    "-video_size",
    `${width}x${height}`,
    "-x",
    `${width / 1.5}`,
    "-y",
    `${height / 1.5}`,
    "-framerate",
    `${fps}`,
    "-i",
    "pipe:0",
  ],
  {
    stdin: "pipe",
  },
);

console.log("Starting ffplay timer...");
console.log(`Resolution: ${width}x${height} @ ${fps}fps`);

// Auto-quit bun process when ffplay exits
ffplay.exited.then(() => {
  console.log("ffplay exited, cleaning up...");
  cleanup();
});

const interval = setInterval(async () => {
  try {
    // Render raw RGBA frame
    const frame = await renderer.render(createFrame(), {
      width,
      height,
      format: "raw",
    });

    ffplay.stdin.write(frame);
  } catch (error) {
    console.error("Error rendering frame:", error);
    cleanup();
  }
}, 1000 / fps);

// Cleanup on exit
function cleanup() {
  clearInterval(interval);
  ffplay.stdin.end();
  ffplay.kill();
  process.exit(0);
}

process.on("SIGINT", cleanup);
process.on("SIGTERM", cleanup);

function createFrame(time = Date.now()): AnyNode {
  // Calculate hue rotation based on time for visible smooth color animation
  const hue = ((time / 1000) * 36) % 360; // Rotate through full color spectrum every 10 seconds
  const angle = ((time / 1000) * 10) % 360; // Rotate gradient angle every 36 seconds

  // Vibrant chroma gradient using HSL colors with good saturation
  const color1 = `hsl(${hue}, 80%, 45%)`; // Saturated color
  const color2 = `hsl(${(hue + 120) % 360}, 80%, 55%)`; // Complementary brighter color
  const color3 = `hsl(${(hue + 240) % 360}, 80%, 35%)`; // Third color

  return container({
    tw: "w-full h-full relative bg-gray-950",
    style: {
      backgroundImage: `linear-gradient(${angle}deg, ${color1} 0%, ${color2} 50%, ${color3} 100%)`,
    },
    children: [
      // Text content
      container({
        tw: "relative w-full h-full flex items-center justify-center backdrop-blur-xl",
        children: [
          text({
            tw: "text-white text-7xl font-semibold font-mono text-shadow-lg",
            text: formatTime(time),
          }),
        ],
      }),
    ],
  });
}

// Format time with milliseconds
function formatTime(timestamp: number): string {
  const date = new Date(timestamp);
  const hours = String(date.getHours()).padStart(2, "0");
  const minutes = String(date.getMinutes()).padStart(2, "0");
  const seconds = String(date.getSeconds()).padStart(2, "0");
  const milliseconds = String(date.getMilliseconds()).padStart(3, "0");
  return `${hours}:${minutes}:${seconds}.${milliseconds}`;
}
