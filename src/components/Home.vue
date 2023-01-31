<script setup lang="ts">
import { onMounted, reactive } from 'vue';
import { ChannelService } from '../services/channelService';
import { useRouter } from "vue-router"
import ChannelTile from './ChannelTile.vue'
import { Channel } from '../../shared/channel';

interface IHomeState {
  channels: Array<Channel>
}

const electron: any = (window as any).electronAPI;
const router = useRouter();
const state: IHomeState = reactive({
    channels: []
});
onMounted(async () => {
    ChannelService.channels = await electron.getCache();
    if (ChannelService.channels.length == 0)
        router.replace("/setup");
    else {
        state.channels = ChannelService.channels.slice(0,10);
    }
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
                <div v-for="channel in state.channels" class="col-xl-3 col-lg-4 col-md-4" >
                    <ChannelTile :data="channel"></ChannelTile>
                </div>
            </div>
        </div>
    </div>



</template>