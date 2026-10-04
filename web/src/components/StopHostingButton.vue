<script setup lang="ts">
import { NPopconfirm } from 'naive-ui'
import type { ButtonProps } from 'naive-ui'
import { stopHosting } from '@/api'
import Notify from '@/notify'
import IconButton from '@/components/IconButton.vue'
import Tooltip from '@/components/Tooltip.vue'
import { Dismiss16Filled } from '@vicons/fluent'

const props = defineProps<{
  hostId: string
  size?: ButtonProps['size']
}>()

const emit = defineEmits<{
  stopped: []
}>()

async function stop() {
  try {
    await stopHosting(props.hostId)
    Notify.success({ content: 'Sharing stopped successfully.', duration: 3000 })
    emit('stopped')
  } catch (err) {
    Notify.error(err instanceof Error ? err.message : 'Failed to stop sharing.')
  }
}
</script>

<template>
  <NPopconfirm
    placement="bottom"
    positive-text="Stop sharing"
    negative-text="Cancel"
    @positive-click="stop"
  >
    <template #trigger>
      <div>
        <Tooltip>
          <template #trigger="{ props }">
            <IconButton v-bind="props" class="danger">
              <Dismiss16Filled />
            </IconButton>
          </template>
          Stop sharing
        </Tooltip>
      </div>
    </template>
    Are you sure you want to stop sharing this item?
  </NPopconfirm>
</template>
