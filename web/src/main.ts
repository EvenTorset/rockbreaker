import { createApp } from 'vue'
import 'easy-tooltips/styles.css'
import 'easy-tooltips'
import App from './App.vue'
import router from './router'
import './main.scss'

createApp(App).use(router).mount('#app')
