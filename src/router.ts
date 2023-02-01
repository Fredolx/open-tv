import { createRouter, createWebHistory } from 'vue-router'
import Setup from './components/Setup.vue'
import Home from './components/Home.vue'
import Settings from './components/Settings.vue'

const routes = [
  { path: '/', name: "Home", component: Home },
  { path: '/setup', name: "Setup", component: Setup },
  { path: '/settings', name: 'settings', component: Settings}
]

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes, // short for `routes: routes`
})

export default router