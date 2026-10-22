<script setup lang="ts" generic="T">
import { provide, computed } from 'vue'

const props = defineProps<{
  nullable?: boolean
  dimUnselected?: boolean
}>()

const modelValue = defineModel<T | null>({ required: true })

const selectValue = (val: T) => {
  if (modelValue.value === val) {
    if (props.nullable) {
      modelValue.value = null
    }
  } else {
    modelValue.value = val
  }
}

const isSelected = (val: T) => modelValue.value === val

provide('radio-group', {
  nullable: !!props.nullable,
  dimUnselected: !!props.dimUnselected,
  selectedValue: computed(() => modelValue.value),
  selectValue,
  isSelected,
})
</script>

<template>
  <slot></slot>
</template>
