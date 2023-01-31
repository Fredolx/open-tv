<script setup lang="ts">
import { ChannelService } from '../services/channelService';
import { useRouter } from 'vue-router';
const electron: any = (window as any).electronAPI;
const router = useRouter();
async function SelectFile() {
    ChannelService.channels = await electron.selectFile();
    if (ChannelService.channels.length > 0)
        router.replace("/");
}
</script>
<template>
    <div class="container text-center mt-5">
        <h3>Welcome to Open-TV</h3>
        <h5>Please select a m3u file to begin</h5>
        <div class="mt-4">
            <button class="btn btn-primary" style="font-size: 1.2em;" @click="SelectFile">Select a file</button>
        </div>
    </div>
</template>