import { Channel } from "./channel";

export class Cache {
    channels!: Array<Channel>;
    url?: string;
    username?: string;
    password?: string;
}