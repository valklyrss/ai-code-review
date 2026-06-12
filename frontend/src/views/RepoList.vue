<template>
  <h1 class="page-title">仓库配置</h1>
  <div class="toolbar"><el-button type="primary" @click="openCreate">新增仓库</el-button></div>
  <div class="panel">
    <el-table :data="repos" v-loading="loading">
      <el-table-column prop="repo_name" label="仓库名称" min-width="120" />
      <el-table-column prop="repo_url" label="仓库地址" min-width="280" show-overflow-tooltip />
      <el-table-column prop="auth_type" label="认证" width="80" />
      <el-table-column prop="branch_pattern" label="分支规则" />
      <el-table-column prop="scan_interval_seconds" label="间隔" width="80" />
      <el-table-column prop="owner_email" label="负责人邮箱" min-width="160" />
      <el-table-column label="启用" width="80"><template #default="{row}"><el-tag :type="row.enabled ? 'success' : 'info'">{{ row.enabled ? '是' : '否' }}</el-tag></template></el-table-column>
      <el-table-column label="操作" width="300">
        <template #default="{row}">
          <el-button size="small" @click="openEdit(row)">编辑</el-button>
          <el-button size="small" @click="test(row)">测试</el-button>
          <el-button size="small" @click="scan(row)">扫描</el-button>
          <el-button size="small" type="danger" @click="remove(row)">删除</el-button>
        </template>
      </el-table-column>
    </el-table>
  </div>
  <el-dialog v-model="visible" :title="editing ? '编辑仓库' : '新增仓库'" width="720px">
    <RepoForm v-model="form" />
    <template #footer><el-button @click="visible=false">取消</el-button><el-button type="primary" @click="save">保存</el-button></template>
  </el-dialog>
</template>
<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { api, Repo } from '../api'
import RepoForm from './RepoForm.vue'

const repos = ref<Repo[]>([])
const loading = ref(false)
const visible = ref(false)
const editing = ref<Repo | null>(null)
const form = ref<any>({})

async function load() { loading.value = true; try { repos.value = await api.repos() } finally { loading.value = false } }
function openCreate() { editing.value = null; form.value = { auth_type: 'SSH', branch_pattern: '*', scan_interval_seconds: 60, enabled: true }; visible.value = true }
function openEdit(row: Repo) { editing.value = row; form.value = { ...row, enabled: !!row.enabled }; visible.value = true }
async function save() { editing.value ? await api.updateRepo(editing.value.id, form.value) : await api.createRepo(form.value); visible.value = false; await load() }
async function test(row: Repo) { const res: any = await api.testRepo(row.id); ElMessage.success(`连接成功，发现 ${res.branches?.length || 0} 个分支`) }
async function scan(row: Repo) { await api.scanRepo(row.id); ElMessage.success('已触发扫描') }
async function remove(row: Repo) { await ElMessageBox.confirm(`删除仓库 ${row.repo_name}?`); await api.deleteRepo(row.id); await load() }
onMounted(load)
</script>

