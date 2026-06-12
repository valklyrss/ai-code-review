<template>
  <h1 class="page-title">AI 设置</h1>

  <div class="panel" v-loading="loading">
    <div class="toolbar">
      <el-select v-model="presetName" placeholder="选择预设 AI 接口" style="width: 260px" @change="applyPreset">
        <el-option v-for="preset in presets" :key="preset.name" :label="preset.name" :value="preset.name" />
      </el-select>
      <el-tag type="info">扫描任务会使用这里保存的配置</el-tag>
    </div>

    <el-form :model="form" label-width="150px" style="max-width: 820px">
      <el-form-item label="配置名称">
        <el-input v-model="form.name" placeholder="例如 DeepSeek 生产配置" />
      </el-form-item>
      <el-form-item label="Provider">
        <el-input v-model="form.provider" disabled />
      </el-form-item>
      <el-form-item label="Base URL">
        <el-input v-model="form.base_url" placeholder="https://api.deepseek.com/v1" />
      </el-form-item>
      <el-form-item label="API Key">
        <el-input
          v-model="form.api_key"
          type="password"
          show-password
          placeholder="留空表示不修改已有密钥"
        />
        <div v-if="maskedKey" class="hint">当前已保存：{{ maskedKey }}</div>
      </el-form-item>
      <el-form-item label="模型">
        <el-input v-model="form.model" placeholder="deepseek-chat" />
      </el-form-item>
      <el-form-item label="超时时间">
        <el-input-number v-model="form.timeout_seconds" :min="10" :max="600" />
      </el-form-item>
      <el-form-item label="Temperature">
        <el-input-number v-model="form.temperature" :min="0" :max="2" :step="0.1" />
      </el-form-item>
      <el-form-item label="Max Tokens">
        <el-input-number v-model="form.max_tokens" :min="256" :max="32768" :step="256" />
      </el-form-item>
      <el-form-item label="启用">
        <el-switch v-model="form.enabled" />
      </el-form-item>
      <el-form-item>
        <el-button type="primary" @click="save">保存 AI 设置</el-button>
      </el-form-item>
    </el-form>
  </div>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import { ElMessage } from 'element-plus'
import { api } from '../api'

interface Preset {
  name: string
  provider: string
  base_url: string
  model: string
}

const loading = ref(false)
const presets = ref<Preset[]>([])
const presetName = ref('')
const maskedKey = ref('')
const form = reactive({
  name: '默认配置',
  provider: 'openai-compatible',
  base_url: '',
  api_key: '',
  model: '',
  timeout_seconds: 120,
  temperature: 0.2,
  max_tokens: 4096,
  enabled: true
})

async function load() {
  loading.value = true
  try {
    const res: any = await api.aiSettings()
    presets.value = res.presets || []
    if (res.setting) {
      Object.assign(form, {
        name: res.setting.name,
        provider: res.setting.provider,
        base_url: res.setting.base_url,
        api_key: '',
        model: res.setting.model,
        timeout_seconds: res.setting.timeout_seconds,
        temperature: res.setting.temperature,
        max_tokens: res.setting.max_tokens,
        enabled: !!res.setting.enabled
      })
      maskedKey.value = res.setting.api_key_masked || ''
    }
  } finally {
    loading.value = false
  }
}

function applyPreset(name: string) {
  const preset = presets.value.find(item => item.name === name)
  if (!preset) return
  form.name = preset.name
  form.provider = preset.provider
  form.base_url = preset.base_url
  form.model = preset.model
}

async function save() {
  await api.saveAiSettings(form)
  ElMessage.success('AI 设置已保存，后续扫描会使用该配置')
  form.api_key = ''
  await load()
}

onMounted(load)
</script>

<style scoped>
.hint {
  margin-top: 6px;
  color: #6b7280;
  font-size: 12px;
}
</style>
