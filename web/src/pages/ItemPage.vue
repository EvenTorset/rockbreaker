<script setup lang="ts">
import { onBeforeUnmount, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { fetchItem, type ItemDetail } from '@/api'
import { useLoader } from '@/composables/useLoader'
import { itemLocation } from '@/util/urls'
import DirectoryView from '@/components/DirectoryView.vue'
import FileView from '@/components/FileView.vue'
import PageState from '@/components/PageState.vue'

const DEFAULT_TITLE = 'Rockbreaker'

const route = useRoute()
const router = useRouter()
const { data: item, loading, error, reload } = useLoader(() => fetchItem(route.path))

watch(() => route.path, () => {
  if (route.name === 'item') reload()
}, { immediate: true })

function titleOf(item: ItemDetail) {
  if (item.kind === 'file' && item.subpath === '') return item.root_name

  const root = item.kind === 'collection' ? '~' : item.root_name
  const path = [root, ...item.subpath.split('/').filter(Boolean)].join('/')
  return item.is_dir ? `${path}/` : path
}

watch(item, (item) => {
  document.title = item ? `${titleOf(item)} - ${DEFAULT_TITLE}` : DEFAULT_TITLE
}, { immediate: true })

onBeforeUnmount(() => {
  document.title = DEFAULT_TITLE
})

function onRenamed(newId: string) {
  if (item.value) router.replace(itemLocation(newId, item.value.subpath))
}
</script>

<template>
  <PageState :loading="loading && !item" :error>
    <template v-if="item">
      <FileView v-if="!item.is_dir" :item @stopped="reload" @renamed="onRenamed" />
      <DirectoryView v-else :item :loading @stopped="reload" @renamed="onRenamed" />
    </template>
  </PageState>
</template>
