import { Channel } from "./channel";
import { Xtream } from "./xtream";

export class Cache {
    channels!: Array<Channel>;
    xtream?: Xtream
    m3u_url?: string
}