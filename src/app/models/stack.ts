import { Node } from "./node";

export class Stack {
  private nodes: Node[] = [];

  constructor() {}

  add(node: Node): void {
    this.nodes.push(node);
  }

  pop(): Node {
    if (this.nodes.length === 0) {
      throw new Error("Stack is empty");
    }
    return this.nodes.pop()!;
  }

  get(): Node | undefined {
    return this.nodes[this.nodes.length - 1];
  }

  hasNodes(): boolean {
    return this.nodes.length > 0;
  }

  clear(): Node | undefined {
    const first = this.nodes[0];
    this.nodes = [];
    return first;
  }
}
