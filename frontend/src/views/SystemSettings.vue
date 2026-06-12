<template>
  <h1 class="page-title">系统设置</h1>
  <div class="panel" v-loading="loading">
    <el-tabs>
      <el-tab-pane label="扫描设置">
        <el-form :model="scanner" label-width="180px" style="max-width: 760px">
          <el-form-item label="定时扫描间隔(秒)">
            <el-input-number v-model="scanner.interval_seconds" :min="5" />
          </el-form-item>
          <el-form-item label="最大并发任务数">
            <el-input-number v-model="scanner.max_concurrent_tasks" :min="1" />
            <div class="hint">当前版本建议保持 1；修改后重启服务可完全生效。</div>
          </el-form-item>
          <el-form-item label="任务最大 diff 行数">
            <el-input-number v-model="scanner.max_diff_lines" :min="1" />
          </el-form-item>
          <el-form-item label="单文件最大 diff 行数">
            <el-input-number v-model="scanner.max_file_diff_lines" :min="1" />
          </el-form-item>
          <el-form-item label="Git 命令超时(秒)">
            <el-input-number v-model="scanner.git_command_timeout_seconds" :min="5" />
          </el-form-item>
          <el-form-item>
            <el-button type="primary" @click="saveScanner">保存扫描设置</el-button>
          </el-form-item>
        </el-form>
      </el-tab-pane>

      <el-tab-pane label="邮箱设置">
        <el-form :model="mail" label-width="150px" style="max-width: 820px">
          <el-form-item label="启用邮件">
            <el-switch v-model="mail.enabled" />
          </el-form-item>
          <el-form-item label="SMTP 地址">
            <el-input v-model="mail.smtp_host" />
          </el-form-item>
          <el-form-item label="SMTP 端口">
            <el-input-number v-model="mail.smtp_port" :min="1" :max="65535" />
          </el-form-item>
          <el-form-item label="用户名">
            <el-input v-model="mail.username" />
          </el-form-item>
          <el-form-item label="密码">
            <el-input v-model="mail.password" type="password" show-password placeholder="留空表示不修改已有密码" />
            <div v-if="mailPasswordMasked" class="hint">当前已保存：{{ mailPasswordMasked }}</div>
          </el-form-item>
          <el-form-item label="发件人">
            <el-input v-model="mail.from_addr" placeholder="AI代码审核 <ai-review@example.com>" />
          </el-form-item>
          <el-form-item label="TLS">
            <el-switch v-model="mail.use_tls" />
          </el-form-item>
          <el-form-item>
            <el-button type="primary" @click="saveMail">保存邮箱设置</el-button>
          </el-form-item>
        </el-form>
      </el-tab-pane>

      <el-tab-pane label="审核规则">
        <el-form :model="review" label-width="160px">
          <el-form-item label="Prompt 模板">
            <el-input v-model="review.default_prompt_name" style="max-width: 360px" />
          </el-form-item>
          <el-form-item label="严重等级">
            <el-input v-model="reviewText.serious_levels" type="textarea" :rows="3" />
          </el-form-item>
          <el-form-item label="允许审核后缀">
            <el-input v-model="reviewText.allowed_extensions" type="textarea" :rows="5" />
          </el-form-item>
          <el-form-item label="忽略路径">
            <el-input v-model="reviewText.ignore_paths" type="textarea" :rows="5" />
          </el-form-item>
          <el-form-item label="忽略后缀">
            <el-input v-model="reviewText.ignore_extensions" type="textarea" :rows="5" />
          </el-form-item>
          <el-form-item>
            <el-button type="primary" @click="saveReview">保存审核规则</el-button>
          </el-form-item>
        </el-form>
      </el-tab-pane>
    </el-tabs>
  </div>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import { ElMessage } from 'element-plus'
import { api } from '../api'

const loading = ref(false)
const mailPasswordMasked = ref('')
const scanner = reactive({
  interval_seconds: 60,
  max_concurrent_tasks: 1,
  max_diff_lines: 3000,
  max_file_diff_lines: 800,
  git_command_timeout_seconds: 120
})
const mail = reactive({
  enabled: false,
  smtp_host: '',
  smtp_port: 465,
  username: '',
  password: '',
  from_addr: 'AI代码审核 <ai-review@example.com>',
  use_tls: true
})
const review = reactive({
  default_prompt_name: 'java_legacy'
})
const reviewText = reactive({
  serious_levels: '',
  allowed_extensions: '',
  ignore_paths: '',
  ignore_extensions: ''
})

async function load() {
  loading.value = true
  try {
    const res: any = await api.settings()
    Object.assign(scanner, res.scanner)
    Object.assign(mail, {
      enabled: !!res.mail.enabled,
      smtp_host: res.mail.smtp_host || '',
      smtp_port: res.mail.smtp_port,
      username: res.mail.username || '',
      password: '',
      from_addr: res.mail.from_addr,
      use_tls: !!res.mail.use_tls
    })
    mailPasswordMasked.value = res.mail.password_masked || ''
    review.default_prompt_name = res.review.default_prompt_name
    reviewText.serious_levels = toText(res.review.serious_levels)
    reviewText.allowed_extensions = toText(res.review.allowed_extensions)
    reviewText.ignore_paths = toText(res.review.ignore_paths)
    reviewText.ignore_extensions = toText(res.review.ignore_extensions)
  } finally {
    loading.value = false
  }
}

async function saveScanner() {
  await api.saveScannerSettings(scanner)
  ElMessage.success('扫描设置已保存')
}

async function saveMail() {
  await api.saveMailSettings(mail)
  ElMessage.success('邮箱设置已保存')
  mail.password = ''
  await load()
}

async function saveReview() {
  await api.saveReviewSettings({
    default_prompt_name: review.default_prompt_name,
    serious_levels: fromText(reviewText.serious_levels),
    allowed_extensions: fromText(reviewText.allowed_extensions),
    ignore_paths: fromText(reviewText.ignore_paths),
    ignore_extensions: fromText(reviewText.ignore_extensions)
  })
  ElMessage.success('审核规则已保存')
}

function toText(value: string[]) {
  return (value || []).join('\n')
}

function fromText(value: string) {
  return value.split(/\r?\n|,/).map(item => item.trim()).filter(Boolean)
}

onMounted(load)
</script>

<style scoped>
.hint {
  margin-left: 12px;
  color: #6b7280;
  font-size: 12px;
}
</style>
