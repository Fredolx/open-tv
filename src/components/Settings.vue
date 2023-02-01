<script setup lang="ts">
import { onMounted, reactive } from 'vue';
import { useRouter } from "vue-router"

const electron: any = (window as any).electronAPI;
const state = reactive({
    loading: false
})
const router = useRouter();

async function deleteCache() {
    state.loading = true;
    await electron.deleteCache();
    state.loading = false;
    router.replace("/setup");
}

function goHome() {
    router.replace("/");
}

</script>
<template>
    <img src="../assets/arrow-left-circle.svg" class="arrow" @click="goHome()" />
    <div class="container text-center">
        <h2 class="mt-3">Settings</h2>
        <button :disabled="state.loading" @click="deleteCache" class="btn btn-danger mt-3">Delete channels
            cache</button>
    </div>
</template>
<style scoped>
.arrow {
    position: absolute; 
    top: 2%; 
    left: 2%; 
    height: 2.5rem;
    cursor: pointer;
    filter: invert(100%) sepia(100%) saturate(0%) hue-rotate(325deg) brightness(101%) contrast(102%);
}
</style>