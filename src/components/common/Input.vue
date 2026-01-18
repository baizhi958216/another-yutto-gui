<script lang="ts" setup>
const props = defineProps<{
  modelValue: string | number
  modelModifiers?: {
    number?: boolean
    trim?: boolean
  }
  placeholder?: string
  type?: string
  disabled?: boolean
  readonly?: boolean
  min?: string
  max?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string | number]
}>()

function handleInput(event: Event) {
  let value = (event.target as HTMLInputElement).value
  if (props.modelModifiers?.trim) {
    value = value.trim()
  }
  if (props.modelModifiers?.number) {
    const parsed = Number(value)
    emit('update:modelValue', Number.isNaN(parsed) ? value : parsed)
    return
  }
  emit('update:modelValue', value)
}
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
    class="input-base"
    @input="handleInput"
  >
</template>
