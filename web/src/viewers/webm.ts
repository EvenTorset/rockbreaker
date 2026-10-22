import { hasPrefix } from './sniff'
import type { ViewerFile } from './types'

const EBML_HEADER = [0x1a, 0x45, 0xdf, 0xa3]
const SEGMENT = 0x18538067
const TRACKS = 0x1654ae6b
const TRACK_ENTRY = 0xae
const TRACK_TYPE = 0x83

const VIDEO = 1

const READ_BYTES = 16 * 1024

interface Vint {
  value: number
  length: number
  unknown: boolean
}

function readVint(bytes: Uint8Array, position: number, isId: boolean): Vint | null {
  if (position >= bytes.length || bytes[position] === 0) return null

  const length = Math.clz32(bytes[position]) - 23
  if (length > 8 || position + length > bytes.length) return null

  const markerMask = 0xff >> length
  let value = isId ? bytes[position] : bytes[position] & markerMask
  let allOnes = (bytes[position] & markerMask) === markerMask

  for (let i = 1; i < length; i++) {
    value = value * 256 + bytes[position + i]
    if (bytes[position + i] !== 0xff) allOnes = false
  }

  return { value, length, unknown: !isId && allOnes }
}

function walk(bytes: Uint8Array, start: number, end: number, types: number[]) {
  let position = start

  while (position < end) {
    const id = readVint(bytes, position, true)
    const size = id && readVint(bytes, position + id.length, false)
    if (!id || !size) return

    const body = position + id.length + size.length
    const bodyEnd = size.unknown ? end : body + size.value

    if (id.value === SEGMENT || id.value === TRACKS || id.value === TRACK_ENTRY) {
      walk(bytes, body, Math.min(bodyEnd, end), types)
      // Nothing after the tracks matters.
      if (id.value === TRACKS) return
    } else if (id.value === TRACK_TYPE && !size.unknown && body + size.value <= bytes.length) {
      let type = 0
      for (let i = 0; i < size.value; i++) type = type * 256 + bytes[body + i]
      types.push(type)
    }

    if (size.unknown) return
    position = bodyEnd
  }
}

const cache = new WeakMap<ViewerFile, Promise<boolean | null>>()

export function hasVideoTrack(file: ViewerFile): Promise<boolean | null> {
  let result = cache.get(file)

  if (!result) {
    result = file.readSlice(0, READ_BYTES).then((bytes) => {
      if (!hasPrefix(bytes, EBML_HEADER)) return null

      const types: number[] = []
      walk(bytes, 0, bytes.length, types)
      return types.length === 0 ? null : types.includes(VIDEO)
    })
    cache.set(file, result)
  }

  return result
}
