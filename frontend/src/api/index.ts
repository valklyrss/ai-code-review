import axios from 'axios'
import { ElMessage } from 'element-plus'

export const http = axios.create({ baseURL: '/api', timeout: 30000 })

http.interceptors.response.use(
  res => res.data,
  err => {
    const msg = err.response?.data?.error || err.message || '请求失败'
    ElMessage.error(msg)
    return Promise.reject(err)
  }
)

export interface Repo {
  id: string
  repo_name: string
  repo_url: string
  auth_type: string
  username?: string
  access_token?: string
  branch_pattern?: string
  scan_interval_seconds?: number
  enabled: number
  owner_email?: string
  sync_status: string
  sync_progress: number
  sync_message?: string
  sync_started_at?: string
  sync_finished_at?: string
}

export const api = {
  repos: () => http.get<unknown, Repo[]>('/repos'),
  createRepo: (data: Partial<Repo>) => http.post<unknown, Repo>('/repos', data),
  updateRepo: (id: string, data: Partial<Repo>) => http.put<unknown, Repo>(`/repos/${id}`, data),
  deleteRepo: (id: string) => http.delete(`/repos/${id}`),
  testRepo: (id: string) => http.post(`/repos/${id}/test`),
  syncRepo: (id: string) => http.post(`/repos/${id}/sync`),
  scanRepo: (id: string) => http.post(`/repos/${id}/scan`),
  tasks: (params: any) => http.get('/tasks', { params }),
  task: (id: string) => http.get(`/tasks/${id}`),
  retryTask: (id: string) => http.post(`/tasks/${id}/retry`),
  issues: (params: any) => http.get('/issues', { params }),
  updateIssueStatus: (id: string, status: string) => http.put(`/issues/${id}/status`, { status }),
  health: () => http.get('/system/health'),
  configSummary: () => http.get('/system/config-summary')
}

