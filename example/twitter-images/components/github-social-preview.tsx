import { readFile } from "node:fs/promises";

export const persistentImages = [
  {
    src: "takumi.svg",
    data: await readFile("../../assets/images/takumi.svg"),
  },
];

export const name = "github-social-preview";

// GitHub social preview canonical size: 1280×640
export const width = 1280;
export const height = 640;

export const fonts = ["geist/Geist[wght].woff2"];

const accentRed = "#ff3535";

export default function GithubSocialPreview() {
  return (
    <div
      style={{
        backgroundColor: "#09090b",
        width: "100%",
        height: "100%",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        fontFamily: "Geist, sans-serif",
        position: "relative",
        overflow: "hidden",
      }}
    >
      {/* Ambient glow outer */}
      <div
        style={{
          position: "absolute",
          inset: 0,
          backgroundImage:
            "radial-gradient(circle at 50% 50%, rgba(255, 53, 53, 0.10) 0%, transparent 70%)",
        }}
      />
      {/* Ambient glow inner hot-spot */}
      <div
        style={{
          position: "absolute",
          inset: 0,
          backgroundImage:
            "radial-gradient(circle at 50% 50%, rgba(255, 53, 53, 0.07) 0%, transparent 40%)",
        }}
      />
      {/* Corner brightening — top-left */}
      <div
        style={{
          position: "absolute",
          inset: 0,
          backgroundImage:
            "radial-gradient(circle at 0% 0%, rgba(255,255,255,0.015) 0%, transparent 50%)",
        }}
      />
      {/* Corner brightening — bottom-right */}
      <div
        style={{
          position: "absolute",
          inset: 0,
          backgroundImage:
            "radial-gradient(circle at 100% 100%, rgba(255,255,255,0.012) 0%, transparent 50%)",
        }}
      />

      {/* Centre lock-up — absolutely centred, safe zone ~560×560 */}
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "center",
          gap: "2.5rem",
          zIndex: 1,
          position: "relative",
        }}
      >
        {/* Logo mark */}
        <div
          style={{
            width: "10rem",
            height: "10rem",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            position: "relative",
          }}
        >
          {/* Glow ring behind logo */}
          <div
            style={{
              position: "absolute",
              width: "11rem",
              height: "11rem",
              borderRadius: "50%",
              backgroundImage:
                "radial-gradient(circle, rgba(255, 53, 53, 0.18) 0%, transparent 70%)",
            }}
          />
          <img
            src={persistentImages[0]?.src}
            alt="Takumi"
            style={{
              width: "10rem",
              height: "10rem",
              position: "relative",
            }}
          />
        </div>

        {/* Wordmark */}
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            gap: "1rem",
          }}
        >
          <span
            style={{
              fontSize: "6.5rem",
              fontWeight: 800,
              color: "#ffffff",
              letterSpacing: "-0.04em",
              lineHeight: 1,
            }}
          >
            Takumi
          </span>
          <span
            style={{
              fontSize: "1.75rem",
              fontWeight: 500,
              color: "rgba(255, 255, 255, 0.45)",
              letterSpacing: "0.01em",
              lineHeight: 1,
            }}
          >
            JSX → PNG · GIF · Video
          </span>
        </div>

        {/* Accent pill */}
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: "0.625rem",
            backgroundColor: "rgba(255, 53, 53, 0.08)",
            border: "1px solid rgba(255, 53, 53, 0.22)",
            borderRadius: "100px",
            padding: "0.6rem 1.4rem",
          }}
        >
          <div
            style={{
              width: 8,
              height: 8,
              borderRadius: "50%",
              backgroundColor: accentRed,
            }}
          />
          <span
            style={{
              fontSize: "1.25rem",
              fontWeight: 600,
              color: accentRed,
              letterSpacing: "0.02em",
            }}
          >
            Built on Rust · Runs Everywhere
          </span>
        </div>
      </div>
    </div>
  );
}
