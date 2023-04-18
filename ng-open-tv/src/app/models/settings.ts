import { StreamCachingEnum } from "./stream-caching-enum";

export class Settings {
    recordingPath?: string;
    useStreamCaching?: StreamCachingEnum;
    cacheSecs?: number;
}