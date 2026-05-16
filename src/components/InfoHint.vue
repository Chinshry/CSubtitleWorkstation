<script setup lang="ts">
defineProps<{
  title: string
  command?: string
  body?: string
  items?: string[]
  placement?: 'left' | 'right' | 'center'
}>()
</script>

<template>
  <span
    class="hint rich-hint"
    :class="{
      'tip-left': placement === 'left',
      'tip-right': placement === 'right',
    }"
    tabindex="0"
    :aria-label="title"
  >
    <span class="rich-hint-card" role="tooltip">
      <strong>{{ title }}</strong>
      <code v-if="command">{{ command }}</code>
      <span v-if="body" class="rich-hint-body">{{ body }}</span>
      <span v-if="items?.length" class="rich-hint-list">
        <span v-for="item in items" :key="item">{{ item }}</span>
      </span>
    </span>
  </span>
</template>
