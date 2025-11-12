declare type PlaygroundOptions = {
  /**
   * @description width of the render viewport.
   * @default 1200
   */
  width?: number;
  /**
   * @description height of the render viewport.
   * @default 630
   */
  height?: number;
  /**
   * @description format to render.
   * @default png
   */
  format?: "png" | "jpeg" | "webp";
  /**
   * @description quality of jpeg format (0-100).
   * @default 75
   */
  quality?: number;
};
