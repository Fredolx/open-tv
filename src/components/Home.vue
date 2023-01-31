<script setup lang="ts">
import { onMounted } from 'vue';
import { ChannelService } from '../services/channelService';
import { useRouter } from "vue-router"
import { channel } from 'diagnostics_channel';

const electron: any = (window as any).electronAPI;
const router = useRouter();
onMounted(async () => {
    ChannelService.channels = await electron.getCache();
    if (ChannelService.channels.length == 0)
        router.replace("/setup");
})
</script>
<template>
    <div v-if="ChannelService.channels.length > 0">
        <img src="../assets/cog.svg" class="settings" />
        <div class="container" style="margin-top: 5rem;">
            <div class="row mb-5">
                <div class="mx-auto col-xl-6 col-lg-6 col-md-8 col-12">
                    <input type="text" style="width: 100%;" class="form-control" placeholder="Type to search...">
                </div>
            </div>

            <div class="row gy-3">

            </div>
        </div>
    </div>



</template>