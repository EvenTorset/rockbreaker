<script setup lang="ts">
import { ref, watch } from 'vue'
import MarkdownIt from 'markdown-it'
import { rawUrl } from '@/util/urls'
import { decodeText } from '../../sniff'
import type { ViewerFile } from '../../types'
import ViewerState from '../../ViewerState.vue'
import RadioGroup from '@/components/RadioGroup.vue'
import RadioButton from '@/components/RadioButton.vue'

const MAX_BYTES = 2 * 1024 * 1024

// Raw HTML in the file is escaped rather than rendered, and markdown-it refuses `javascript:` links, so what someone
// shared can't run anything on this page.
const markdown = new MarkdownIt({ html: false, linkify: true })

// What the render call passes along to the rules below.
type Env = { resolveImage?: (src: string) => string } & Record<string, unknown>

const renderImage = markdown.renderer.rules.image!
markdown.renderer.rules.image = (tokens, index, options, env: Env | undefined, self) => {
  const token = tokens[index]
  const src = token.attrGet('src')
  if (src && env?.resolveImage) token.attrSet('src', env.resolveImage(String(src)))
  return renderImage(tokens, index, options, env ?? {}, self)
}

const renderLinkOpen = markdown.renderer.rules.link_open
  ?? ((tokens, index, options, _env, self) => self.renderToken(tokens, index, options))
markdown.renderer.rules.link_open = (tokens, index, options, env: Env | undefined, self) => {
  const token = tokens[index]
  if (/^(https?:)?\/\//i.test(String(token.attrGet('href') ?? ''))) {
    token.attrSet('target', '_blank')
    token.attrSet('rel', 'noopener noreferrer')
  }
  return renderLinkOpen(tokens, index, options, env ?? {}, self)
}

/** Joins a relative path onto a folder, or returns `null` if it climbs out of the shared item. */
function joinPath(directory: string, relative: string) {
  const parts = relative.startsWith('/') ? [] : directory.split('/').filter(Boolean)

  for (const part of relative.split('/')) {
    if (part === '' || part === '.') continue
    if (part === '..') {
      if (parts.pop() === undefined) return null
    } else {
      parts.push(part)
    }
  }
  return parts.join('/')
}

function safeDecode(text: string) {
  try {
    return decodeURIComponent(text)
  } catch {
    return text
  }
}

const props = defineProps<{
  file: ViewerFile
}>()

const source = ref('')
const html = ref('')
/** Whether the text of the file is shown as it is, instead of what it makes. */
const showRaw = ref(false)
const loading = ref(true)
const error = ref<string | null>(null)

/** Images next to the markdown file are pointed at the shared files, as they wouldn't be found otherwise. */
function resolveImage(file: ViewerFile, src: string) {
  const isAbsolute = /^[a-z][a-z0-9+.-]*:/i.test(src) || src.startsWith('//') || src.startsWith('#')
  if (isAbsolute || file.directory === null) return src

  const [path] = src.split(/[?#]/)
  const target = joinPath(file.directory, safeDecode(path))
  return target ? `${rawUrl(file.itemId, target)}?inline=1` : src
}

watch(() => props.file, async (file) => {
  loading.value = true
  error.value = null

  try {
    source.value = decodeText(await file.readSlice(0, MAX_BYTES))
    html.value = markdown.render(source.value, { resolveImage: (src: string) => resolveImage(file, src) } satisfies Env)
  } catch (err) {
    if (!file.signal.aborted) error.value = err instanceof Error ? err.message : 'Could not read the file.'
  } finally {
    loading.value = false
  }
}, { immediate: true })
</script>

<template>
  <ViewerState :loading :error>
    <div class="view-mode" role="group" aria-label="How to show the file">
      <RadioGroup v-model="showRaw">
        <RadioButton :value="false">Rendered</RadioButton>
        <RadioButton :value="true">Raw</RadioButton>
      </RadioGroup>
    </div>

    <pre v-if="showRaw" class="raw" tabindex="0">{{ source }}</pre>
    <article v-else class="markdown" v-html="html"></article>
  </ViewerState>
</template>

<style lang="scss" scoped>

.view-mode {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 8px 12px;
  border-bottom: 1px solid var(--color-2);
}

.raw {
  margin: 0;
  padding: 16px 20px;
  color: var(--color-6);
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, 'Liberation Mono', monospace;
  font-size: 13px;
  line-height: 1.5;
  tab-size: 4;
  // The lines of prose are long, so they wrap instead of making the page scroll sideways.
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}

.markdown {
  padding: 20px 28px 28px;
  color: var(--color-5);
  font-size: 15px;
  line-height: 1.65;
  overflow-wrap: anywhere;

  :deep(> :first-child) {
    margin-top: 0;
  }

  :deep(> :last-child) {
    margin-bottom: 0;
  }

  :deep(h1),
  :deep(h2),
  :deep(h3),
  :deep(h4),
  :deep(h5),
  :deep(h6) {
    margin: 1.6em 0 0.5em;
    color: var(--color-6);
    font-weight: 600;
    line-height: 1.3;
  }

  :deep(h1) {
    font-size: 1.9em;
    padding-bottom: 0.3em;
    border-bottom: 1px solid var(--color-2);
  }

  :deep(h2) {
    font-size: 1.45em;
    padding-bottom: 0.25em;
    border-bottom: 1px solid var(--color-2);
  }

  :deep(h3) {
    font-size: 1.2em;
  }

  :deep(h4),
  :deep(h5),
  :deep(h6) {
    font-size: 1em;
  }

  :deep(p),
  :deep(ul),
  :deep(ol),
  :deep(blockquote),
  :deep(pre),
  :deep(table) {
    margin: 0 0 1em;
  }

  :deep(ul),
  :deep(ol) {
    padding-left: 1.6em;
  }

  :deep(li + li) {
    margin-top: 0.25em;
  }

  :deep(a) {
    color: var(--color-accent);
  }

  :deep(blockquote) {
    padding: 0 1em;
    color: var(--color-4);
    border-left: 3px solid var(--color-3);
  }

  :deep(code) {
    padding: 0.15em 0.4em;
    background-color: var(--color-1);
    border-radius: 4px;
    font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, 'Liberation Mono', monospace;
    font-size: 0.9em;
  }

  :deep(pre) {
    padding: 14px 16px;
    overflow-x: auto;
    background-color: var(--color-1);
    border-radius: 6px;
    line-height: 1.5;

    code {
      padding: 0;
      background: none;
      font-size: 13px;
      white-space: pre;
    }
  }

  :deep(img) {
    max-width: 100%;
  }

  :deep(hr) {
    margin: 1.6em 0;
    border: 0;
    border-top: 1px solid var(--color-2);
  }

  :deep(table) {
    display: block;
    max-width: 100%;
    overflow-x: auto;
    border-collapse: collapse;
  }

  :deep(th),
  :deep(td) {
    padding: 6px 12px;
    border: 1px solid var(--color-2);
  }

  :deep(th) {
    background-color: var(--color-1);
    color: var(--color-6);
  }
}

</style>
