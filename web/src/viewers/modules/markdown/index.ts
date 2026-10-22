import { defineViewer } from '../../types'

const EXTENSIONS = new Set(['md', 'markdown', 'mdown', 'mkd', 'mkdn'])
const MAX_BYTES = 2 * 1024 * 1024

export default defineViewer({
  id: 'markdown',
  priority: 10,
  test: file => EXTENSIONS.has(file.extension) && (file.size === null || file.size <= MAX_BYTES),
  load: () => import('./MarkdownViewer.vue'),
})
