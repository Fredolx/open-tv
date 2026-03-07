import { MediaType } from "./mediaType";
import { ViewMode } from "./viewMode";

export interface SidebarNavEvent {
  viewType: ViewMode;
  mediaTypeFilter?: MediaType[];
  sourceFilter?: number[];
}
