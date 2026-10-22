import type { ItemDetail } from '@/api'
import type { ViewerFile } from './types'

const HEADER_LENGTH = 4096

async function readUpTo(response: Response, start: number, end: number) {
  if (!response.body) {
    return new Uint8Array(await response.arrayBuffer()).slice(start, end)
  }

  const reader = response.body.getReader()
  const chunks: Uint8Array[] = []
  let length = 0

  while (length < end) {
    const { done, value } = await reader.read()
    if (done) break
    chunks.push(value)
    length += value.length
  }
  void reader.cancel().catch(() => {})

  const all = new Uint8Array(length)
  let offset = 0
  for (const chunk of chunks) {
    all.set(chunk, offset)
    offset += chunk.length
  }
  return all.slice(start, end)
}

function createReader(url: string, size: number | null, signal: AbortSignal) {
  let header: Promise<Uint8Array> | undefined

  async function fetchRange(start: number, end: number): Promise<Uint8Array> {
    const last = size === null ? end : Math.min(end, size)
    if (last <= start) return new Uint8Array()

    const response = await fetch(url, { headers: { Range: `bytes=${start}-${last - 1}` }, signal })
    if (response.status === 416) return new Uint8Array()
    if (!response.ok) throw new Error(`Could not read the file (${response.status}).`)

    if (response.status === 206) return new Uint8Array(await response.arrayBuffer())
    return readUpTo(response, start, last)
  }

  return async function readSlice(start: number, end: number): Promise<Uint8Array> {
    if (end <= HEADER_LENGTH) {
      header ??= fetchRange(0, HEADER_LENGTH)
      return (await header).slice(start, end)
    }

    return fetchRange(start, end)
  }
}

export function createViewerFile(item: ItemDetail, signal: AbortSignal): ViewerFile {
  const name = item.file_name
  const dot = name.lastIndexOf('.')

  return {
    itemId: item.id,
    name,
    extension: dot > 0 ? name.slice(dot + 1).toLowerCase() : '',
    size: item.file_size,
    directory: item.subpath === '' ? null : item.parent_subpath ?? '',
    downloadUrl: item.download_url,
    previewUrl: `${item.download_url}?inline=1`,
    readSlice: createReader(item.download_url, item.file_size, signal),
    signal,
  }
}
