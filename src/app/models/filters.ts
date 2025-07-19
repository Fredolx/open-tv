import { MediaType } from "./mediaType";
import { SortType } from "./sortType";
import { ViewMode } from "./viewMode";

export class Filters {
  public query?: string;
  public source_ids!: number[];
  public media_types!: MediaType[];
  public view_type!: ViewMode;
  public page!: number;
  public group_id?: number;
  public series_id?: number;
  public use_keywords!: boolean;
  public sort?: SortType;
  public season?: number;
}
