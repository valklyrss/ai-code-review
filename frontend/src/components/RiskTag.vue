<template>
  <el-tag :type="type">{{ value || 'INFO' }}</el-tag>
</template>
<script setup lang="ts">
const props = defineProps<{ value?: string }>()
const type = props.value === 'CRITICAL' || props.value === 'HIGH' ? 'danger' : props.value === 'MEDIUM' ? 'warning' : 'info'
</script>
