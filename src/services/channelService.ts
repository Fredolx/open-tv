import { reactive } from 'vue'
import { Channel } from '../../shared/channel'

interface IChannelService {
  channels: Array<Channel>,
  channelStarting: boolean
}

export const ChannelService: IChannelService = reactive({
  channels: [],
  channelStarting: false
})