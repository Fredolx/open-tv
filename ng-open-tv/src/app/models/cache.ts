import { Channel } from "./channel";
import { Xtream } from "./xtream";

export class Cache {
    name!: string;
    channels!: Array<Channel>;
    xtream?: Xtream;
    url?: string;
}