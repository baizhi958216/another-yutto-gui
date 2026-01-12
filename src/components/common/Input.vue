<script lang="ts" setup>
defineProps<{
  modelValue: string | number
  placeholder?: string
  type?: string
  disabled?: boolean
  readonly?: boolean
  min?: string
  max?: string
}>()

defineEmits<{
  'update:modelValue': [value: string]
}>()
</script>

<template>
  <input
    :value="modelValue"
    :type="type || 'text'"
    :placeholder="placeholder"
    :disabled="disabled"
    :readonly="readonly"
    :min="min"
    :max="max"
    class="input"
    @input="$emit('update:modelValue', ($event.target as HTMLInputElement).value)"
  >
</template>
