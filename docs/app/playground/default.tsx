export default function Image() {
  return (
    <div
      style={{
        justifyContent: "center",
        alignItems: "center",
        flexDirection: "column",
        width: "100%",
        height: "100%",
        backgroundColor: "white",
      }}
    >
      <h1
        style={{
          fontWeight: 500,
          fontSize: "4rem",
          display: "block",
        }}
      >
        Welcome to{" "}
        <span
          style={{
            color: "red",
          }}
        >
          Takumi{" "}
        </span>
        Playground!
      </h1>
      <span
        style={{
          color: "rgb(0 0 0 60%)",
          fontWeight: 350,
          fontSize: "2.5rem",
          display: "block",
        }}
      >
        You can try out and experiment with Takumi here.
      </span>
    </div>
  );
}

export const options: PlaygroundOptions = {
  width: 1200,
  height: 630,
  format: "png",
};
