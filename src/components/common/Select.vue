<script lang="ts" setup>
defineProps<{
  modelValue: string | number
  options: Array<{ label: string, value: string | number }>
  disabled?: boolean
}>()

defineEmits<{
  'update:modelValue': [value: string | number]
}>()
</script>

<template>
  <select
    :value="modelValue"
    :disabled="disabled"
    class="select"
    @change="$emit('update:modelValue', ($event.target as HTMLSelectElement).value)"
  >
    <option v-for="option in options" :key="option.value" :value="option.value">
      {{ option.label }}
    </option>
  </select>
</template>
