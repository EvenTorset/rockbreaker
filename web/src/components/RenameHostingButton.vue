<script setup lang="ts">
import { computed, ref } from 'vue'
import { NButton, NInput, NModal } from 'naive-ui'
import type { ButtonProps } from 'naive-ui'
import { renameHosting } from '@/api'
import Notify from '@/notify'
import { isValidHostId } from '@/util/urls'
import Col from '@/components/Col.vue'
import Row from '@/components/Row.vue'
import IconButton from '@/components/IconButton.vue'
import Tooltip from '@/components/Tooltip.vue'
import { LinkEdit16Filled } from '@vicons/fluent'

const props = defineProps<{
  hostId: string
  size?: ButtonProps['size']
}>()

const emit = defineEmits<{
  renamed: [newId: string]
}>()

const open = ref(false)
const newId = ref('')
const saving = ref(false)

const problem = computed(() => {
  if (newId.value === '' || isValidHostId(newId.value)) return null
  return 'Only letters, numbers, underscores and dashes can be used.'
})
const canSave = computed(() => isValidHostId(newId.value) && newId.value !== props.hostId && !saving.value)

function show() {
  newId.value = props.hostId
  open.value = true
}

async function save() {
  if (!canSave.value) return

  saving.value = true
  try {
    await renameHosting(props.hostId, newId.value)
    Notify.success({ content: `Item renamed to ${newId.value}.`, duration: 3000 })
    open.value = false
    emit('renamed', newId.value)
  } catch (err) {
    Notify.error(err instanceof Error ? err.message : 'Failed to rename the item.')
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <Tooltip>
    <template #trigger="{ props }">
      <IconButton v-bind="props" @click="show">
        <LinkEdit16Filled />
      </IconButton>
    </template>
    Edit ID
  </Tooltip>

  <NModal
    v-model:show="open"
    preset="card"
    title="Edit hosted item ID"
    :style="{ width: 'min(440px, 92vw)' }"
    :mask-closable="!saving"
    :closable="!saving"
  >
    <Col align="stretch" gap="12px">
      <p>
        The links to this item will use the new ID. Links that use the current one stop working.
      </p>
      <NInput
        v-model:value="newId"
        placeholder="e.g. share_custom"
        :status="problem ? 'error' : undefined"
        :disabled="saving"
        autofocus
        @keyup.enter="save"
      />
      <span v-if="problem" class="problem">{{ problem }}</span>
    </Col>

    <template #footer>
      <Row justify="flex-end" gap="8px">
        <NButton :disabled="saving" @click="open = false">Cancel</NButton>
        <NButton class="accent" :disabled="!canSave" :loading="saving" @click="save">Save</NButton>
      </Row>
    </template>
  </NModal>
</template>

<style lang="scss" scoped>

.problem {
  color: var(--color-6);
  font-size: 0.85rem;
}

</style>
