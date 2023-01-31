import { reactive } from 'vue'
import { Channel } from '../../shared/channel'

interface IChannelService {
    channels: Array<Channel>
  }

export const ChannelService: IChannelService = reactive({
    channels: []
})