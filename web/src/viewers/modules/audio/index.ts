import { defineViewer } from '../../types'
import { hasVideoTrack } from '../../webm'

const EXTENSIONS = new Set(['mp3', 'wav', 'ogg', 'oga', 'opus', 'flac', 'm4a', 'aac', 'weba'])

export default defineViewer({
  id: 'audio',
  async test(file) {
    if (EXTENSIONS.has(file.extension)) return true
    return file.extension === 'webm' && await hasVideoTrack(file) === false
  },
  load: () => import('./AudioViewer.vue'),
})
