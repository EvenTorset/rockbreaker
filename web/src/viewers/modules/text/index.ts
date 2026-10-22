import { looksLikeText } from '../../sniff'
import { defineViewer } from '../../types'

export default defineViewer({
  id: 'text',
  priority: -100,
  async test(file) {
    return looksLikeText(await file.readSlice(0, 4096))
  },
  load: () => import('./TextViewer.vue'),
})
