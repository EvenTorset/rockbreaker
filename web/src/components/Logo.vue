<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'

withDefaults(defineProps<{
  weight?: number
}>(), {
  weight: 1
})

const svg = ref<HTMLElement>()
const strokeWidth = ref<number>(10)
let observer: ResizeObserver

onMounted(() => {
  if (!svg.value) return
  observer = new ResizeObserver(([entry]) => {
    const height = entry.borderBoxSize?.[0]?.blockSize || entry.contentRect.height
    if (height > 0) {
      strokeWidth.value = 2400 / height
    }
  })
  observer.observe(svg.value)
})

onBeforeUnmount(() => {
  observer?.disconnect()
})
</script>

<template>
  <div class="logo">
    <svg ref="svg" xmlns="http://www.w3.org/2000/svg" xml:space="preserve" viewBox="0 0 1024 1024">
      <defs>
        <radialGradient id="fillGradient" gradientTransform="translate(-0.5 0) scale(2, 2)">
          <stop offset="0%" stop-color="#4280ff"/>
          <stop offset="8.33%" stop-color="rgba(62, 125, 255, 0.93)"/>
          <stop offset="16.67%" stop-color="rgba(59, 123, 255, 0.86)"/>
          <stop offset="25%" stop-color="rgba(55, 120, 255, 0.79)"/>
          <stop offset="33.33%" stop-color="rgba(51, 118, 255, 0.72)"/>
          <stop offset="41.67%" stop-color="rgba(48, 115, 255, 0.65)"/>
          <stop offset="50%" stop-color="rgba(44, 112, 255, 0.57)"/>
          <stop offset="58.33%" stop-color="rgba(40, 109, 255, 0.5)"/>
          <stop offset="66.67%" stop-color="rgba(35, 107, 255, 0.43)"/>
          <stop offset="75%" stop-color="rgba(31, 104, 255, 0.36)"/>
          <stop offset="83.33%" stop-color="rgba(26, 101, 255, 0.29)"/>
          <stop offset="100%" stop-color="rgba(15, 95, 255, 0.15)"/>
        </radialGradient>

        <radialGradient id="strokeGradient" gradientTransform="translate(-0.5 -1) scale(2, 2)">
          <stop offset="0%" stop-color="#4280ff"/>
          <stop offset="8.33%" stop-color="rgba(62, 125, 255, 0.93)"/>
          <stop offset="16.67%" stop-color="rgba(59, 123, 255, 0.86)"/>
          <stop offset="25%" stop-color="rgba(55, 120, 255, 0.79)"/>
          <stop offset="33.33%" stop-color="rgba(51, 118, 255, 0.72)"/>
          <stop offset="41.67%" stop-color="rgba(48, 115, 255, 0.65)"/>
          <stop offset="50%" stop-color="rgba(44, 112, 255, 0.57)"/>
          <stop offset="58.33%" stop-color="rgba(40, 109, 255, 0.5)"/>
          <stop offset="66.67%" stop-color="rgba(35, 107, 255, 0.43)"/>
          <stop offset="75%" stop-color="rgba(31, 104, 255, 0.36)"/>
          <stop offset="83.33%" stop-color="rgba(26, 101, 255, 0.29)"/>
          <stop offset="100%" stop-color="rgba(15, 95, 255, 0.15)"/>
        </radialGradient>

        <path id="shape" d="M114.8 554.9 277.2 190l397.2-41.8 234.8 323.1-162.4 365-397.2 41.8zm191.1-325.3L163.3 549.8l206.1 283.6 348.7-36.6 142.6-320.3-206.1-283.6zM674.2 148.8l234.4 322.7-162.2 364.3-396.7 41.7-234.4-322.6 162.2-364.4zM369.2 834l.6-.1 348.1-36.6.6-.1.2-.5L861 477l.2-.5-.3-.5-205.7-283.2-.3-.5-.6.1L306.2 229l-.6.1-.2.5L163 549.4l-.2.5.3.5 205.7 283.2zm305.5-686.2-397.8 41.8L114.2 555l235.1 323.6 397.8-41.8 162.7-365.4zM369.6 832.9 163.9 549.8 306.3 230l348.1-36.6 205.7 283.2-142.4 319.7zm-246.1-69.6c-15.8 0-30.8-7.6-40.1-20.4-16-22.1-11.1-53.1 11-69.1 8.5-6.2 18.6-9.5 29.1-9.5 15.8 0 30.8 7.6 40.1 20.4 7.8 10.7 10.9 23.8 8.8 36.8-2.1 13.1-9.1 24.5-19.8 32.3-8.6 6.3-18.6 9.5-29.1 9.5M234.1 542.4l31.3-70.3 66.2-7 74.2 60.1 18.9-23.3-82.7-67-62.9 6.6 68.7-154.3L625.7 258 724 393.3l-60.7 6.4L595 484l22.5 18.3 60.7-75 65.4-6.9 46.2 63.6-113.6 255.2-277.9 29.2zM403 631.2l44.7 36.2 35.6-44 44 35.6 35.6-44 44 35.6 36.2-44.7-44.7-36.2-35.6 44-44-35.6-35.6 44-44-35.6zm87.2-115.7 24.1 19.5 19.5-24.1-24.1-19.5zm-6.7-63.7 24.1 19.5 19.5-24.1-24.1-19.5zm-6.7-63.6 24.1 19.5 19.5-24.1-24.1-19.5zM625.4 258.4l97.7 134.4-20.2 2.1L664 399l-.5.1h-.4l-.3.3-67.8 83.8-.6.8.8.6 21.8 17.6.8.6.6-.8 60.3-74.4 64.9-6.8 45.9 63.1-113.6 254.9-277.3 29.1-163.9-225.6 31.1-69.8 65.7-6.9 73.6 59.6.8.6.6-.8 18.3-22.5.6-.8-.8-.6-82.2-66.6-.3-.3h-.4l-.9.1-39.8 4.2-21 2.2 68.2-153.3zM501 408.4l.6-.8 18.9-23.3.6-.8-.8-.6L497 364l-.8-.6-.6.8-18.9 23.3-.6.8.8.6 23.3 18.9zm6.7 63.6.6-.8 18.9-23.3.6-.8-.8-.6-23.3-18.9-.8-.6-.6.8-18.9 23.3-.6.8.8.6 23.3 18.9zm6.7 63.7.6-.8 18.9-23.3.6-.8-.8-.6-23.3-18.9-.8-.6-.6.8-18.9 23.3-.6.8.8.6 23.3 18.9zm48.3 77.3-43.2-35-.8-.6-.6.8-35 43.2-43.2-35-.8-.6-.6.8-35.6 44-.6.8.8.6 44 35.6.8.6.6-.8 35-43.2 43.2 35 .8.6.6-.8 35-43.2 43.2 35 .8.6.6-.8 35.6-44 .6-.8-.8-.6-44-35.6-.8-.6-.6.8zm63.2-355.6-278.5 29.3L278.2 442l22.7-2.4 39.8-4.2.9-.1 82.2 66.6-18.3 22.5-74-59.9-66.6 7-31.6 70.9L398.1 769l278.5-29.3 113.9-255.8-46.6-64.1-65.9 6.9-60.5 74.7-21.8-17.6 67.8-83.7.5-.1 39-4 21.9-2.3zM500.8 407l-23.3-18.9 18.9-23.3 23.3 18.9zm6.7 63.6-23.3-18.9 18.9-23.3 23.3 18.9zm6.7 63.7-23.3-18.9 18.9-23.3 23.3 18.9zm48.7 80.1 35.6-44 44 35.6-35.6 44zl-35.6 44-44-35.6-35.6 44-44-35.6 35.6-44 44 35.6 35.6-44zM143.891 326.895a50 50 0 0 1 0-100 50 50 0 0 1 0 100zM90 447.94a50 50 0 0 1 0-100 50 50 0 0 1 0 100zM123.442 763.82a50 50 0 0 1 0-100 50 50 0 0 1 0 100zM201.324 871.016a50 50 0 0 1 0-100 50 50 0 0 1 0 100zM822.924 255.308a50 50 0 0 1 0-100 50 50 0 0 1 0 100zM900.806 362.503a50 50 0 0 1 0-100 50 50 0 0 1 0 100zM934.017 678.408a50 50 0 0 1 0-100 50 50 0 0 1 0 100zM880.124 799.454a50 50 0 0 1 0-100 50 50 0 0 1 0 100z"/>

        <clipPath id="insideClip">
          <use href="#shape" />
        </clipPath>
      </defs>

      <use href="#shape" fill="url(#fillGradient)" />

      <use
        href="#shape"
        fill="none"
        stroke="url(#strokeGradient)"
        :stroke-width="strokeWidth"
        clip-path="url(#insideClip)"
      />
    </svg>

    <h1 :style="{ fontWeight: `${200 * weight}` }">Rockbreaker</h1>
    <a href="https://github.com/EvenTorset/rockbreaker" target="_blank">GitHub</a>
  </div>
</template>

<style lang="scss" scoped>

.logo {
  position: relative;
  display: flex;
  align-items: center;
  font-size: min(1.5em, 3cqw);
  width: fit-content;
  height: fit-content;

  & > svg {
    fill: var(--color-accent);
    width: 5em;
    height: 5em;
  }

  & > h1 {
    color: var(--color-6);
    font-size: 2.5em;
    text-shadow: 0 0.036em var(--color-0);
    margin: -0.4em 0 0;
    user-select: none;
    cursor: inherit;
    white-space: nowrap;

    :root.light-color-scheme & {
      text-shadow: none;
    }
  }

  & > a {
    position: absolute;
    left: 6.55em;
    bottom: 0.8em;
    color: var(--color-accent);
    font-size: 0.8em;

    &:hover {
      color: var(--color-accent-hover);
    }
  }
}

</style>
