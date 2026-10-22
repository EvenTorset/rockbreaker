<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink, useRouter } from 'vue-router'
import { NBreadcrumb, NBreadcrumbItem } from 'naive-ui'
import { ArrowUp16Filled } from '@vicons/fluent'
import type { ItemDetail } from '@/api'
import { itemLocation } from '@/util/urls'
import IconButton from './IconButton.vue'
import Row from './Row.vue'
import Tooltip from './Tooltip.vue'

const props = defineProps<{
  item: ItemDetail
}>()

const router = useRouter()

const crumbs = computed(() => {
  const parts = props.item.subpath.split('/').filter(Boolean)
  const root = props.item.kind === 'collection' ? 'Shared items' : props.item.root_name

  return [
    { label: root, to: itemLocation(props.item.id, '') },
    ...parts.map((part, index) => ({
      label: part,
      to: itemLocation(props.item.id, parts.slice(0, index + 1).join('/')),
    })),
  ]
})

function goUp() {
  if (props.item.parent_subpath !== null) {
    router.push(itemLocation(props.item.id, props.item.parent_subpath))
  }
}
</script>

<template>
  <Row gap="8px" class="path-bar">
    <Tooltip>
      <template #trigger="{ props: triggerProps }">
        <IconButton
          v-bind="triggerProps"
          aria-label="Go up one level"
          :disabled="item.parent_subpath === null"
          @click="goUp"
        >
          <ArrowUp16Filled />
        </IconButton>
      </template>
      Go up one level
    </Tooltip>

    <NBreadcrumb class="breadcrumb">
      <NBreadcrumbItem v-for="(crumb, index) in crumbs" :key="crumb.to">
        <RouterLink v-if="index < crumbs.length - 1" :to="crumb.to" class="crumb-link">
          {{ crumb.label }}
        </RouterLink>
        <span v-else>{{ crumb.label }}</span>
      </NBreadcrumbItem>
    </NBreadcrumb>
  </Row>
</template>

<style lang="scss" scoped>

.path-bar {
  min-width: 0;
}

.breadcrumb {
  min-width: 0;
  white-space: normal;
  overflow-wrap: anywhere;
}

.crumb-link {
  color: inherit;
  text-decoration: none;
}

</style>
