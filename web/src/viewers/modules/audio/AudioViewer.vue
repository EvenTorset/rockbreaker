<script setup lang="ts">
import { ref } from 'vue'
import Dim from '@/components/Dim.vue'
import type { ViewerFile } from '../../types'
import { useRememberedVolume } from '../../useRememberedVolume'

defineProps<{
  file: ViewerFile
}>()

const failed = ref(false)
const media = ref<HTMLMediaElement>()
const { onVolumeChange } = useRememberedVolume(media)
</script>

<template>
  <div class="audio-viewer">
    <Dim v-if="failed">Your browser can't play this audio. You can still download it.</Dim>
    <audio
      v-else
      ref="media"
      @volumechange="onVolumeChange"
      :src="file.previewUrl"
      controls
      preload="metadata"
      @error="failed = true"
    ></audio>
  </div>
</template>

<style lang="scss" scoped>

.audio-viewer {
  padding: 24px;
  text-align: center;
}

audio {
  width: 100%;
}

</style>
