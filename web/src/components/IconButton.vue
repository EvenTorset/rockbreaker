<script setup lang="ts">
withDefaults(defineProps<{
  tag?: string
}>(), {
  tag: 'button'
})
</script>

<template>
  <component :is="tag" class="n-button icon lite-icon-button" type="button"><slot></slot></component>
</template>

<style>

.lite-icon-button {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  padding: 0;
  margin: 0;
  position: relative;
  cursor: pointer;
  font-family: inherit;
  font-size: 14px;
  user-select: none;

  svg {
    width: 18px;
    height: 18px;
  }
}

</style>
