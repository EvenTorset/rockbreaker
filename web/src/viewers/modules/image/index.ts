import { asciiBytes, hasPrefix } from '../../sniff'
import { defineViewer } from '../../types'

const EXTENSIONS = new Set(['png', 'apng', 'jpg', 'jpeg', 'jpe', 'jfif', 'gif', 'webp', 'avif', 'bmp', 'ico', 'svg'])

const SIGNATURES = [
  [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a], // PNG
  [0xff, 0xd8, 0xff], // JPEG
  asciiBytes('GIF87a'),
  asciiBytes('GIF89a'),
]

export default defineViewer({
  id: 'image',
  async test(file) {
    if (EXTENSIONS.has(file.extension)) return true

    const header = await file.readSlice(0, 12)
    const isWebp = hasPrefix(header, asciiBytes('RIFF')) && hasPrefix(header, asciiBytes('WEBP'), 8)
    return isWebp || SIGNATURES.some(signature => hasPrefix(header, signature))
  },
  metadata(file) {
    if (file.extension === 'svg') return []

    return new Promise((resolve) => {
      const image = new Image()
      const finish = (details: string[]) => {
        image.onload = image.onerror = null
        resolve(details)
      }

      image.onload = () => finish(image.naturalWidth > 0 ? [`${image.naturalWidth} × ${image.naturalHeight} px`] : [])
      image.onerror = () => finish([])
      file.signal.addEventListener('abort', () => finish([]), { once: true })
      image.src = file.previewUrl
    })
  },
  load: () => import('./ImageViewer.vue'),
})
