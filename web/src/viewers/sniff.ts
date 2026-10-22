export function hasPrefix(bytes: Uint8Array, signature: readonly number[], offset = 0) {
  if (bytes.length < offset + signature.length) return false
  return signature.every((byte, index) => bytes[offset + index] === byte)
}

export function asciiBytes(text: string) {
  return Array.from(text, char => char.charCodeAt(0))
}

export function looksLikeText(bytes: Uint8Array) {
  if (bytes.length === 0) return false

  let controls = 0
  for (const byte of bytes) {
    if (byte === 0) return false
    if (byte < 0x20 && byte !== 0x09 && byte !== 0x0a && byte !== 0x0c && byte !== 0x0d && byte !== 0x1b) controls++
  }
  if (controls / bytes.length > 0.01) return false

  try {
    new TextDecoder('utf-8', { fatal: true }).decode(bytes, { stream: true })
    return true
  } catch {
    return false
  }
}

export function decodeText(bytes: Uint8Array) {
  return new TextDecoder('utf-8').decode(bytes, { stream: true })
}
