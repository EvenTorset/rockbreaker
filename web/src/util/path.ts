export function escapeRegExp(text: string) {
  return text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}


export function matchRanges(text: string, pattern: string | RegExp | null): [number, number][] {
  if (pattern === null || pattern === '') return []

  const regex = typeof pattern === 'string'
    ? new RegExp(escapeRegExp(pattern), 'gi')
    : new RegExp(pattern.source, pattern.flags.includes('g') ? pattern.flags : `${pattern.flags}g`)

  const ranges: [number, number][] = []
  for (const match of text.matchAll(regex)) {
    if (match[0].length > 0) ranges.push([match.index, match.index + match[0].length])
  }
  return ranges
}
