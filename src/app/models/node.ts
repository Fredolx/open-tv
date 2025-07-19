import { NodeType } from "./nodeType";

export class Node {
  readonly id: number;
  readonly name: string;
  readonly type: NodeType;
  readonly scrollPosition: number;
  query?: string;
  page?: number;

  constructor(id: number, name: string, type: NodeType, query?: string) {
    this.id = id;
    this.name = name;
    this.type = type;
    this.query = query;
    this.scrollPosition = window.scrollY;
  }

  toString(): string {
    return `Viewing: ${this.name}`;
  }
}
