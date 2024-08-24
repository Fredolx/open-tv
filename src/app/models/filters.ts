import { MediaType } from "./mediaType";
import { ViewMode } from "./viewMode";

export class Filters {
    public query?: string;
    public source_ids!: number[];
    public media_types!: MediaType[];
    public view_type!: ViewMode
    public page!: number;
    public group_name?: string;
    public series_id?: number;
}
