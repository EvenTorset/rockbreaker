import { defineViewer } from '../../types'
import { hasVideoTrack } from '../../webm'

const EXTENSIONS = new Set(['mp4', 'm4v', 'webm', 'ogv', 'mov'])

export default defineViewer({
  id: 'video',
  async test(file) {
    if (!EXTENSIONS.has(file.extension)) return false
    return file.extension !== 'webm' || await hasVideoTrack(file) !== false
  },
  load: () => import('./VideoViewer.vue'),
})
