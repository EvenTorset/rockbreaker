<script setup lang="ts">
import { NSwitch } from 'naive-ui'
import Row from './Row.vue'

defineProps<{
  title: string
  description: string
  value: boolean
  disabled?: boolean
  loading?: boolean
}>()

defineEmits<{
  'update:value': [value: boolean]
}>()
</script>

<template>
  <Row justify="space-between" gap="16px">
    <div>
      <h3>{{ title }}</h3>
      <p>{{ description }}</p>
    </div>
    <NSwitch
      :value
      :disabled
      :loading
      @update:value="$emit('update:value', $event)"
    />
  </Row>
</template>
