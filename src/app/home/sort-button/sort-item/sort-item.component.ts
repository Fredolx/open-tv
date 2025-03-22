import { Component, Input } from "@angular/core";
import { SortType, getSortTypeText } from "../../../models/sortType";
import { MemoryService } from "../../../memory.service";

@Component({
  selector: "app-sort-item",
  templateUrl: "./sort-item.component.html",
  styleUrl: "./sort-item.component.css",
})
export class SortItemComponent {
  constructor(public memory: MemoryService) {}

  @Input()
  sortType?: SortType;

  getText(): String {
    return getSortTypeText(this.sortType);
  }

  notifySortChange() {
    this.memory.Sort.next([this.sortType!, true]);
  }
}
