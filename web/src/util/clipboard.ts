import Notify from '@/notify'

export async function copyText(text: string, label: string) {
  try {
    await navigator.clipboard.writeText(text)
    Notify.success({ content: `Copied ${label} to clipboard!`, duration: 3000 })
  } catch {
    Notify.error(`Could not copy ${label} to the clipboard.`)
  }
}
