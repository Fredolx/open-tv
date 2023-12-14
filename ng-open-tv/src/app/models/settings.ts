import { Source } from "./source";

export class Settings {
    recordingPath?: string;
    useStreamCaching?: boolean;
    mpvParams?: string;
    sources?: Source[];
}