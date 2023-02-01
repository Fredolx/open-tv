<script setup lang="ts">
import { ChannelService } from '../services/channelService';
import { useRouter } from 'vue-router';
import { reactive } from 'vue';
const electron: any = (window as any).electronAPI;
const router = useRouter();
const setupState = reactive({
    loading: false
});
async function SelectFile() {
    setupState.loading = true;
    ChannelService.channels = await electron.selectFile();
    setupState.loading = false;
    if (ChannelService.channels.length > 0)
        router.replace("/");
}
</script>
<template>
    <div class="container text-center mt-5">
        <h3>Welcome to Open-TV</h3>
        <h5>Please select a m3u file to begin</h5>
        <div class="mt-4">
            <button class="btn btn-primary" :disabled="setupState.loading" style="font-size: 1.2em;" @click="SelectFile">Select a file</button>
        </div>
    </div>
</template>