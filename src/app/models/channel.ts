import { MediaType } from "./mediaType";

export class Channel {
    id!: number;
    name!: string;
    group?: string;
    image?: string;
    url!: string;
    media_type!: MediaType;
    source_id!: number;
    favorite!: boolean
}