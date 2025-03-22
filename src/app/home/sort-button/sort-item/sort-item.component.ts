import { Component, Input } from "@angular/core";
import { SortType } from "../../../models/sortType";
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

  getSortTypeText(): String {
    switch (this.sortType) {
      case SortType.alphabeticalAscending:
        return "Sort alphabetically asc";
      case SortType.alphabeticalDescending:
        return "Sort alphabetically desc";
      case SortType.provider:
        return "Sort provider";
    }
    return "";
  }

  notifySortChange() {
    this.memory.Sort.next(this.sortType!);
  }
}
