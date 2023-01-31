import { createApp } from 'vue'
import "./style.css"
import App from './App.vue'
import router from './router'
import { VTooltip } from 'floating-vue'
import 'floating-vue/dist/style.css'

const app = createApp(App)

app.use(router)
app.directive('tooltip', VTooltip)
app.mount('#app')
