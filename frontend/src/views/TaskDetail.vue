<template>
  <h1 class="page-title">任务详情</h1>
  <div v-if="data.task" class="panel">
    <el-descriptions :column="2" border>
      <el-descriptions-item label="仓库">{{ data.task.repo_name }}</el-descriptions-item>
      <el-descriptions-item label="分支">{{ data.task.branch_name }}</el-descriptions-item>
      <el-descriptions-item label="状态"><StatusTag :value="data.task.status" /></el-descriptions-item>
      <el-descriptions-item label="风险"><RiskTag :value="data.task.risk_level" /></el-descriptions-item>
      <el-descriptions-item label="old">{{ data.task.old_commit_id }}</el-descriptions-item>
      <el-descriptions-item label="new">{{ data.task.new_commit_id }}</el-descriptions-item>
      <el-descriptions-item label="邮件">{{ data.task.email_sent ? '已发送' : '未发送' }}</el-descriptions-item>
      <el-descriptions-item label="错误">{{ data.task.error_msg || '-' }}</el-descriptions-item>
    </el-descriptions>

    <el-tabs v-model="activeTab" style="margin-top:16px">
      <el-tab-pane label="问题" name="issues">
        <el-table :data="data.issues">
          <el-table-column label="级别" width="110">
            <template #default="{ row }"><RiskTag :value="row.issue_level" /></template>
          </el-table-column>
          <el-table-column prop="file_path" label="文件" min-width="220" show-overflow-tooltip />
          <el-table-column prop="line_no" label="行" width="80" />
          <el-table-column prop="title" label="标题" min-width="180" />
          <el-table-column prop="description" label="说明" min-width="260" show-overflow-tooltip />
          <el-table-column label="状态" width="170">
            <template #default="{ row }">
              <el-select v-model="row.status" @change="(s: string) => update(row, s)">
                <el-option v-for="s in statuses" :key="s" :value="s" :label="s" />
              </el-select>
            </template>
          </el-table-column>
        </el-table>
      </el-tab-pane>

      <el-tab-pane label="Commit" name="commits">
        <el-table :data="data.commits">
          <el-table-column prop="commit_id" label="Commit" min-width="260" />
          <el-table-column prop="author_name" label="作者" />
          <el-table-column prop="author_email" label="邮箱" min-width="180" />
          <el-table-column prop="commit_msg" label="消息" min-width="240" />
        </el-table>
      </el-tab-pane>

      <el-tab-pane label="文件" name="files">
        <el-table :data="data.files">
          <el-table-column prop="file_path" label="文件" min-width="260" show-overflow-tooltip />
          <el-table-column prop="change_type" label="类型" width="100" />
          <el-table-column prop="skipped" label="跳过" width="100" />
          <el-table-column prop="skip_reason" label="原因" min-width="180" />
        </el-table>
      </el-tab-pane>
    </el-tabs>
  </div>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import { useRoute } from 'vue-router'
import { api } from '../api'
import StatusTag from '../components/StatusTag.vue'
import RiskTag from '../components/RiskTag.vue'

const route = useRoute()
const data = reactive<any>({})
const activeTab = ref('issues')
const statuses = ['TODO', 'CONFIRMED', 'FIXED', 'FALSE_POSITIVE', 'IGNORED']

async function load() {
  Object.assign(data, await api.task(route.params.id as string))
}

async function update(row: any, status: string) {
  await api.updateIssueStatus(row.id, status)
}

onMounted(load)
</script>

