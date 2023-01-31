<script setup lang="ts">
import { reactive } from 'vue';
import { Channel } from '../../shared/channel';
import { ChannelService } from '../services/channelService';
const props = defineProps({
    data: Channel
});
const electron: any = (window as any).electronAPI;
const state = reactive({
    show: true,
    starting: false
})
function getClass() {
    return {
        'starting': state.starting,
        'disabled': !state.starting && ChannelService.channelStarting
    }
}
async function click() {
    if(ChannelService.channelStarting === true)
        return;
    state.starting = true;
    ChannelService.channelStarting = true;
    await electron.playChannel(props.data?.url);
    state.starting = false;
    ChannelService.channelStarting = false;
}
</script>
<template>
    <div v-tooltip="data?.name" v-bind:class="getClass()" @click="click" class="channel d-inline-flex p-1">
        <img @error="state.show = false" class="channel-image" v-if="data?.image && state.show" :src="data?.image">
        <div class="channel-title my-auto">{{ data?.name }}</div>
    </div>
</template>

<style scoped>
.disabled {
    opacity: 20%;
}
.channel {
    border-radius: 0.25rem;
    background-color: #343a40;
    height: 3em;
    width: 100%;
    overflow-y: hidden;
}

.channel-title {
    font-size: 0.9rem;
    margin-left: 10px;
    margin-right: 10px;
}

.channel-image {
    width: 75px;
    height: auto;
    object-fit: fill
}

.channel:hover {
    background: #007bff;
    cursor: pointer;
}

.starting {
    background: #0048ff;
    animation-name: pulse;
    animation-timing-function: ease-in-out;
    animation-iteration-count: infinite;
    animation-duration: 2s;
}

.starting:hover {
    background: #0048ff;
}

@keyframes pulse {
    from {
        transform: scale3d(1, 1, 1);
    }

    50% {
        transform: scale3d(1.05, 1.05, 1.05);
    }

    to {
        transform: scale3d(1, 1, 1);
    }
}
</style>