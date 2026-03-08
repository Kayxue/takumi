export default function AnimatedShowcase() {
  return (
    <div
      tw="flex h-full w-full items-center justify-center overflow-hidden bg-[#fff7ed]"
      style={{
        backgroundImage: "linear-gradient(180deg, #fff7ed 0%, #fee2e2 100%)",
      }}
    >
      <div
        className="cube"
        tw="flex h-32 w-32 items-center justify-center rounded-2xl bg-amber-300 font-semibold text-xl text-orange-950"
      >
        Animated!
      </div>
    </div>
  );
}

export const options: PlaygroundOptions = {
  width: 640,
  height: 360,
  stylesheets: [
    `
      .cube {
        animation: stretch-cube 3000ms cubic-bezier(0.65, 0, 0.35, 1) infinite;
        transform-origin: center;
      }

      @keyframes stretch-cube {
        0% {
          transform: rotate(0deg) scale(1, 1);
          border-radius: 16px;
        }

        25% {
          transform: rotate(-3deg) scale(1.08, 0.92);
          border-radius: 28px 18px 24px 14px;
        }

        50% {
          transform: rotate(0deg) scale(0.94, 1.06);
          border-radius: 50%;
        }

        75% {
          transform: rotate(3deg) scale(1.04, 0.96);
          border-radius: 14px 26px 18px 30px;
        }

        100% {
          transform: rotate(0deg) scale(1, 1);
          border-radius: 16px;
        }
      }
    `,
  ],
  animation: {
    durationMs: 3000,
    fps: 24,
    format: "webp",
  },
};
