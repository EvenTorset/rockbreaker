<script setup lang="ts">
import { NButton, NCard, NIcon } from 'naive-ui'
import { Document24Regular } from '@vicons/fluent'
import type { ItemDetail } from '@/api'
import Col from './Col.vue'
import Dim from './Dim.vue'
import Row from './Row.vue'
import RenameHostingButton from './RenameHostingButton.vue'
import StopHostingButton from './StopHostingButton.vue'

defineProps<{
  item: ItemDetail
}>()

const emit = defineEmits<{
  stopped: []
  renamed: [newId: string]
}>()
</script>

<template>
  <NCard class="file-view" title="File">
    <template #header-extra>
      <Row v-if="item.is_host">
        <RenameHostingButton :host-id="item.id" @renamed="emit('renamed', $event)" />
        <StopHostingButton :host-id="item.id" @stopped="emit('stopped')" />
      </Row>
    </template>
    <Col gap="12px">
      <NIcon :size="48" :component="Document24Regular" />
      <h1>{{ item.file_name }}</h1>
      <Dim>Size: {{ item.file_size }}</Dim>
      <Row justify="center" gap="8px" wrap class="actions">
        <NButton class="accent" tag="a" :href="item.download_url" download>
          Download
        </NButton>
      </Row>
    </Col>
  </NCard>
</template>

<style lang="scss" scoped>

.file-view {
  max-width: 500px;
  margin: 3rem auto;
}

h1 {
  font-size: 1.4rem;
  text-align: center;
  word-break: break-all;
}

.actions {
  margin-top: 12px;
}

</style>
