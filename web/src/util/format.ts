const KB = 1024
const MB = KB * 1024
const GB = MB * 1024

export function formatBytes(bytes: number | null | undefined, fallback = '-'): string {
  if (bytes === null || bytes === undefined) return fallback
  if (bytes >= GB) return `${(bytes / GB).toFixed(2)} GB`
  if (bytes >= MB) return `${(bytes / MB).toFixed(2)} MB`
  if (bytes >= KB) return `${(bytes / KB).toFixed(2)} kB`
  return `${bytes} B`
}

let dateFormat: Intl.DateTimeFormat | undefined

export function formatDateTime(millis: number | null | undefined, fallback = '-'): string {
  if (millis === null || millis === undefined) return fallback
  dateFormat ??= new Intl.DateTimeFormat(undefined, { dateStyle: 'medium', timeStyle: 'short' })
  return dateFormat.format(millis)
}
