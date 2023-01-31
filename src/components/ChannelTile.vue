<script setup lang="ts">
import { reactive } from 'vue';
import { Channel } from '../../shared/channel';
defineProps({
    data: Channel
});
const state = reactive({
    show: true
})
function getClass() {
    return {
        'playing': false
    }
}
function click() {
    return true;
}
</script>
<template>
    <div v-tooltip="data?.name" v-bind:class="getClass()" @click="click" class="channel d-inline-flex">
        <img @error="state.show = false" class="channel-image" v-if="data?.image && state.show" :src="data?.image">
        <div class="channel-title my-auto">{{ data?.name }}</div>
    </div>
</template>

<style scoped>
.channel {
    border-radius: 0.25rem;
    background-color: #343a40;
    height: 3em;
    width: 100%;
    overflow-y: hidden;
}

.channel-title {
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

.playing {
    background: #0048ff;
    animation-name: pulse;
    animation-timing-function: ease-in-out;
    animation-iteration-count: infinite;
    animation-duration: 2s;
}

.playing:hover {
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