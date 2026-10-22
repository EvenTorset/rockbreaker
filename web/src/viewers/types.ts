import type { Component } from 'vue'

/** What a viewer is told about the file it is asked to preview. */
export interface ViewerFile {
  /** The shared item the file belongs to. */
  itemId: string
  /** The file name, like `example.md`. */
  name: string
  /** The lowercase extension without the dot, or an empty string when there isn't one. */
  extension: string
  /** The size in bytes, if it's known. */
  size: number | null
  /**
   * The folder the file is in, as a path inside of the shared item (empty for the top level). This is `null` when the
   * file was shared on its own, since it has no neighbours.
   */
  directory: string | null
  /** A URL that downloads the file. */
  downloadUrl: string
  /** A URL to put in an `<img>`, `<video>` and the like. For anything else than media, use `readSlice`. */
  previewUrl: string
  /**
   * Reads the bytes from `start` up to (not including) `end`. Fewer are returned if the file ends before that.
   */
  readSlice(start: number, end: number): Promise<Uint8Array>
  /** Aborted when the visitor navigates away. Pass it to anything that fetches. */
  signal: AbortSignal
}

export interface Viewer {
  /** Must be unique. */
  id: string
  /**
   * Viewers are tried from the highest priority to the lowest, and the first one that accepts a file is used. Viewers
   * with the same priority are tried in the order they were registered. Defaults to 0.
   */
  priority?: number
  /**
   * Whether this viewer should be used for the file. Look at the name or extension first, and read from the file only
   * when that isn't enough, as every viewer that is tried before one accepts gets to run.
   */
  test: (file: ViewerFile) => boolean | Promise<boolean>
  /**
   * Extra details about the file, like the dimensions of an image. They're added after the size and date under the
   * name of the file. This can take a while, as the preview doesn't wait for it.
   */
  metadata?: (file: ViewerFile) => string[] | Promise<string[]>
  /** Loads the component that renders the preview. It receives the `ViewerFile` as its `file` prop. */
  load: () => Promise<{ default: Component }>
}

/** Only here to get type checking for the definition of a viewer. */
export function defineViewer(viewer: Viewer): Viewer {
  return viewer
}
