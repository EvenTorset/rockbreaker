import type { Component, VNode } from 'vue'

export type SingleOrArray<T> = T | T[]
export type StaticRenderableContent = string | VNode | undefined | null
export type ComponentOrStaticRenderableContent = StaticRenderableContent | Component
export type Renderable = SingleOrArray<
 | ComponentOrStaticRenderableContent
 | (() => ComponentOrStaticRenderableContent)
>
