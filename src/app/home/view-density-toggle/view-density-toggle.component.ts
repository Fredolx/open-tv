import { Component } from "@angular/core";
import { MemoryService } from "../../memory.service";
import { ViewDensity } from "../../models/viewDensity";
import { ViewMode } from "../../models/viewMode";
import { MediaType } from "../../models/mediaType";

@Component({
  selector: "app-view-density-toggle",
  templateUrl: "./view-density-toggle.component.html",
  styleUrl: "./view-density-toggle.component.css",
})
export class ViewDensityToggleComponent {
  ViewDensity = ViewDensity;

  constructor(public memory: MemoryService) {}

  get density(): ViewDensity {
    return this.memory.ViewDensity.value;
  }

  set(density: ViewDensity) {
    this.memory.setViewDensity(density);
    // If switching to list while on dashboard, auto-navigate to All Channels
    if (density === ViewDensity.List && this.memory.ActiveSidebarItem.value === "home") {
      this.memory.ActiveSidebarItem.next("all");
      this.memory.SidebarNav.next({
        viewType: ViewMode.All,
        mediaTypeFilter: [MediaType.livestream, MediaType.movie, MediaType.serie],
      });
    }
  }
}
