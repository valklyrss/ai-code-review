import { createRouter, createWebHistory } from 'vue-router'
import Dashboard from './views/Dashboard.vue'
import RepoList from './views/RepoList.vue'
import TaskList from './views/TaskList.vue'
import TaskDetail from './views/TaskDetail.vue'
import IssueList from './views/IssueList.vue'

export default createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', component: Dashboard },
    { path: '/repos', component: RepoList },
    { path: '/tasks', component: TaskList },
    { path: '/tasks/:id', component: TaskDetail },
    { path: '/issues', component: IssueList }
  ]
})

