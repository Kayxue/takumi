import type {
  ContainerNode,
  ImageNode,
  Node,
  NodeMetadata,
  TextNode,
} from "@takumi-rs/helpers";

export interface FontDetails {
  /**
   * The name of the font. If not provided, the name in the font file will be used.
   */
  name?: string;
  /**
   * The font data.
   */
  data: Uint8Array | ArrayBuffer;
  /**
   * The weight of the font. If not provided, the weight in the font file will be used.
   */
  weight?: number;
  /**
   * The style of the font. If not provided, the style in the font file will be used.
   */
  style?:
    | "normal"
    | "italic"
    | "oblique"
    | `oblique ${number}deg`
    | (string & {});
}

export type Font = FontDetails | Uint8Array | ArrayBuffer;

export type { ContainerNode, ImageNode, Node, NodeMetadata, TextNode };

/**
 * @deprecated Use `Node` instead.
 */
export type AnyNode = Node;

export type Keyframes = Record<string, Record<string, Record<string, unknown>>>;

/**
 * @deprecated use `ImageSource` instead.
 */
export type PersistentImage = ImageSource;
