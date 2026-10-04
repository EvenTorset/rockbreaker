import { createRouter, createWebHistory } from 'vue-router'
import { setHostToken } from '@/util/hostToken'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'dashboard',
      component: () => import('@/pages/DashboardPage.vue'),
    },
    {
      path: '/:id/:subpath(.*)?',
      name: 'item',
      component: () => import('@/pages/ItemPage.vue'),
    },
  ],
})

router.beforeEach((to) => {
  const { token, ...query } = to.query
  if (typeof token !== 'string') return

  setHostToken(token)
  return { path: to.path, query, hash: to.hash, replace: true }
})

export default router
