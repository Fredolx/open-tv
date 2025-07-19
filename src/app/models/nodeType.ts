import { MediaType } from "./mediaType";

export enum NodeType {
  Category = "category",
  Series = "series",
  Season = "season",
}

export function fromMediaType(type: MediaType) {
  switch (type) {
    case MediaType.group:
      return NodeType.Category;
    case MediaType.serie:
      return NodeType.Series;
    case MediaType.season:
      return NodeType.Season;
    default:
      throw new Error("Invalid type: " + MediaType.livestream.toString());
  }
}
