import { NodeType } from "./nodeType";
import { ViewMode } from "./viewMode";

export class Node {
  readonly id: number;
  readonly name: string;
  readonly type: NodeType;
  readonly scrollPosition: number;
  query?: string;
  page?: number;
  fromViewType?: ViewMode;

  constructor(id: number, name: string, type: NodeType, query?: string, fromViewType?: ViewMode) {
    this.id = id;
    this.name = name;
    this.type = type;
    this.query = query;
    this.scrollPosition = window.scrollY;
    this.fromViewType = fromViewType;
  }

  toString(): string {
    return `Viewing: ${this.name}`;
  }
}
