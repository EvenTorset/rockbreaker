interface Key {
  s: string
  digits: boolean
}

const keyCache = new Map<string, Key>()
let keyCacheTimeout: ReturnType<typeof setTimeout> | null

function keyOf(s: string): Key {
  const cached = keyCache.get(s)
  if (cached) return cached

  const lower = s.toLowerCase()
  let digits = false
  for (let i = 0, n = lower.length; i < n; i++) {
    const code = lower.charCodeAt(i)
    if (code >= 48 && code <= 57) {
      digits = true
      break
    }
  }

  const key = { s: lower, digits }
  keyCache.set(s, key)
  return key
}

export function naturalCompare(a: string, b: string): number {
  if (a === b) return 0
  if (a.length === 0 || b.length === 0) return a.length - b.length

  if (keyCacheTimeout === null) {
    keyCacheTimeout = setTimeout(() => {
      keyCacheTimeout = null
      keyCache.clear()
    }, 10_000)
  }

  const ka = keyOf(a)
  const kb = keyOf(b)
  const x = ka.s
  const y = kb.s

  if (!ka.digits && !kb.digits) {
    return x === y ? 0 : x < y ? -1 : 1
  }

  const la = x.length
  const lb = y.length
  let i = 0, j = 0

  while (i < la && j < lb) {
    const ca = x.charCodeAt(i)
    const cb = y.charCodeAt(j)

    if (ca >= 48 && ca <= 57 && cb >= 48 && cb <= 57) {
      let si = i
      while (si < la && x.charCodeAt(si) === 48) si++
      let sj = j
      while (sj < lb && y.charCodeAt(sj) === 48) sj++

      let ei = si
      while (ei < la) {
        const c = x.charCodeAt(ei)
        if (c < 48 || c > 57) break
        ei++
      }
      let ej = sj
      while (ej < lb) {
        const c = y.charCodeAt(ej)
        if (c < 48 || c > 57) break
        ej++
      }

      const na = ei - si
      const nb = ej - sj
      if (na !== nb) return na - nb
      for (let k = 0; k < na; k++) {
        const p = x.charCodeAt(si + k)
        const q = y.charCodeAt(sj + k)
        if (p !== q) return p - q
      }

      const rawa = ei - i
      const rawb = ej - j
      if (rawa !== rawb) return rawa - rawb

      i = ei
      j = ej
      continue
    }

    if (ca !== cb) return ca < cb ? -1 : 1
    i++
    j++
  }

  return (la - i) - (lb - j)
}
