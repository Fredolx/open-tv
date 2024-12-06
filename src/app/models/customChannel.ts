import { Channel } from "./channel";
import { ChannelHeaders } from "./channelHeaders";

export class CustomChannel {
    data!: Channel;
    headers?: ChannelHeaders;
}