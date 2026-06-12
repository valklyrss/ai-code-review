<template>
  <el-form :model="form" label-width="120px">
    <el-form-item label="仓库名称"><el-input v-model="form.repo_name" /></el-form-item>
    <el-form-item label="仓库地址"><el-input v-model="form.repo_url" /></el-form-item>
    <el-form-item label="认证方式">
      <el-radio-group v-model="form.auth_type">
        <el-radio-button label="SSH" />
        <el-radio-button label="HTTP" />
      </el-radio-group>
    </el-form-item>
    <template v-if="form.auth_type === 'HTTP'">
      <el-form-item label="用户名"><el-input v-model="form.username" /></el-form-item>
      <el-form-item label="Access Token"><el-input v-model="form.access_token" type="password" show-password /></el-form-item>
    </template>
    <el-form-item label="分支规则"><el-input v-model="form.branch_pattern" placeholder="dev,master,release-*" /></el-form-item>
    <el-form-item label="扫描间隔"><el-input-number v-model="form.scan_interval_seconds" :min="10" /></el-form-item>
    <el-form-item label="负责人邮箱"><el-input v-model="form.owner_email" /></el-form-item>
    <el-form-item label="启用"><el-switch v-model="form.enabled" /></el-form-item>
  </el-form>
</template>
<script setup lang="ts">
import { reactive, watch } from 'vue'

const props = defineProps<{ modelValue: any }>()
const emit = defineEmits<{ 'update:modelValue': [value: any] }>()
const form = reactive({
  repo_name: '',
  repo_url: '',
  auth_type: 'SSH',
  username: '',
  access_token: '',
  branch_pattern: '*',
  scan_interval_seconds: 60,
  enabled: true,
  owner_email: ''
})
watch(() => props.modelValue, v => Object.assign(form, v || {}), { immediate: true })
watch(form, v => emit('update:modelValue', { ...v }), { deep: true })
</script>

