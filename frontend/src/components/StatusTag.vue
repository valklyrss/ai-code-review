<template>
  <el-tag :type="type">{{ value || '-' }}</el-tag>
</template>
<script setup lang="ts">
const props = defineProps<{ value?: string }>()
const type = props.value === 'SUCCESS' ? 'success' : props.value === 'FAILED' ? 'danger' : props.value === 'RUNNING' ? 'warning' : 'info'
</script>

