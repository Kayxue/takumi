export default function Image() {
  return (
    <div
      style={{
        backgroundImage: "url(https://picsum.photos/1200/630)",
        width: "100%",
        height: "100%",
        justifyContent: "center",
        alignItems: "center",
      }}
    >
      <div
        style={{
          padding: "2rem",
          backgroundColor: "rgb(255 255 255 / 0.5)",
          justifyContent: "center",
          alignItems: "center",
          flexDirection: "column",
        }}
      >
        <h1
          style={{
            marginTop: 0,
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
            marginBottom: 0,
          }}
        >
          You can try out and experiment with Takumi here.
        </span>
      </div>
    </div>
  );
}

export const options: PlaygroundOptions = {
  width: 1200,
  height: 630,
  format: "png",
};
