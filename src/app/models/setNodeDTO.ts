import { NodeType } from "./nodeType";

export class SetNodeDTO {
  public id: number;
  public name: string;
  public type: NodeType;
  public sourceId?: number;

  constructor(id: number, name: string, type: NodeType, sourceId?: number) {
    this.id = id;
    this.name = name;
    this.type = type;
    this.sourceId = sourceId;
  }
}
