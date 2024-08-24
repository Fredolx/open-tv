export enum FocusArea {
    ViewMode,
    Filters,
    Tiles
}

export const FocusAreaPrefix = {
    [FocusArea.ViewMode]: "viewMode-",
    [FocusArea.Filters]: "filter-",
    [FocusArea.Tiles]: "tile-"
  };