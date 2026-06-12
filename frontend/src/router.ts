import { createRouter, createWebHistory } from 'vue-router'
import Dashboard from './views/Dashboard.vue'
import RepoList from './views/RepoList.vue'
import RepoDetail from './views/RepoDetail.vue'
import TaskList from './views/TaskList.vue'
import TaskDetail from './views/TaskDetail.vue'
import IssueList from './views/IssueList.vue'
import AiSettings from './views/AiSettings.vue'

export default createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', component: Dashboard },
    { path: '/repos', component: RepoList },
    { path: '/repos/:id', component: RepoDetail },
    { path: '/tasks', component: TaskList },
    { path: '/tasks/:id', component: TaskDetail },
    { path: '/issues', component: IssueList },
    { path: '/settings/ai', component: AiSettings }
  ]
})
