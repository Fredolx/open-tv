import { Component, EventEmitter, HostBinding, Input, Output } from "@angular/core";
import { Router } from "@angular/router";
import { MemoryService } from "../memory.service";
import { ViewMode } from "../models/viewMode";
import { MediaType } from "../models/mediaType";
import { SidebarNavEvent } from "../models/sidebarNavEvent";
import { Source } from "../models/source";
import { PlayerState } from "../models/playerState";

interface SidebarItem {
  id: string;
  label: string;
  icon: string;
  viewType?: ViewMode;
  mediaTypeFilter?: MediaType[];
}

@Component({
  selector: "app-sidebar",
  templateUrl: "./sidebar.component.html",
  styleUrl: "./sidebar.component.css",
})
export class SidebarComponent {
  @HostBinding('class.collapsed')
  @Input() collapsed = false;
  @Output() toggleCollapse = new EventEmitter<void>();

  navItems: SidebarItem[] = [
    {
      id: "home",
      label: "Home",
      icon: "M10,20V14H14V20H19V12H22L12,3L2,12H5V20H10Z",
      viewType: ViewMode.Home,
    },
    {
      id: "all",
      label: "All Channels",
      icon: "M17,2V4H20.59L14.76,9.83L16.17,11.24L22,5.41V9H24V2M12,11.5A2.5,2.5 0 0,1 14.5,14A2.5,2.5 0 0,1 12,16.5A2.5,2.5 0 0,1 9.5,14A2.5,2.5 0 0,1 12,11.5M6,2A2,2 0 0,0 4,4V20A2,2 0 0,0 6,22H18A2,2 0 0,0 20,20V12L18,14V20H6V4H12L14,2H6Z",
      viewType: ViewMode.All,
      mediaTypeFilter: [MediaType.livestream, MediaType.movie, MediaType.serie],
    },
    {
      id: "live",
      label: "Live TV",
      icon: "M8.5 21H4C2.9 21 2 20.1 2 19V5C2 3.9 2.9 3 4 3H20C21.1 3 22 3.9 22 5V11.5C21.4 11.2 20.7 11 20 11V5H4V19H8.5V21M12.5 17.5C12.5 16.4 12.8 15.4 13.2 14.5H7V16H12.7C12.6 16.5 12.5 17 12.5 17.5M7 12H17V11H7V12M7 8H17V7H7V8M17 17L22 14.5V19.5L17 17Z",
      viewType: ViewMode.All,
      mediaTypeFilter: [MediaType.livestream],
    },
    {
      id: "movies",
      label: "Movies",
      icon: "M18,4L20,8H17L15,4H13L15,8H12L10,4H8L10,8H7L5,4H4A2,2 0 0,0 2,6V18A2,2 0 0,0 4,20H20A2,2 0 0,0 22,18V4H18Z",
      viewType: ViewMode.All,
      mediaTypeFilter: [MediaType.movie],
    },
    {
      id: "series",
      label: "Series",
      icon: "M20,2H8A2,2 0 0,0 6,4V16A2,2 0 0,0 8,18H20A2,2 0 0,0 22,16V4A2,2 0 0,0 20,2M20,16H8V4H20V16M12.5,15L17,11L12.5,7V15M4,6H2V20A2,2 0 0,0 4,22H18V20H4V6Z",
      viewType: ViewMode.All,
      mediaTypeFilter: [MediaType.serie],
    },
  ];

  browseItems: SidebarItem[] = [
    {
      id: "categories",
      label: "Categories",
      icon: "M10 4H4C2.89 4 2 4.89 2 6V18C2 19.1 2.9 20 4 20H20C21.1 20 22 19.1 22 18V8C22 6.89 21.1 6 20 6H12L10 4Z",
      viewType: ViewMode.Categories,
    },
    {
      id: "favorites",
      label: "Favorites",
      icon: "M12,17.27L18.18,21L16.54,13.97L22,9.24L14.81,8.62L12,2L9.19,8.62L2,9.24L7.45,13.97L5.82,21L12,17.27Z",
      viewType: ViewMode.Favorites,
    },
    {
      id: "history",
      label: "History",
      icon: "M13.5,8H12V13L16.28,15.54L17,14.33L13.5,12.25V8M13,3A9,9 0 0,0 4,12H1L4.96,16.03L9,12H6A7,7 0 0,1 13,5A7,7 0 0,1 20,12A7,7 0 0,1 13,19C11.07,19 9.32,18.21 8.06,16.94L6.64,18.36C8.27,20 10.5,21 13,21A9,9 0 0,0 22,12A9,9 0 0,0 13,3",
      viewType: ViewMode.History,
    },
    {
      id: "hidden",
      label: "Hidden",
      icon: "M11.83,9L15,12.16C15,12.11 15,12.05 15,12A3,3 0 0,0 12,9C11.94,9 11.89,9 11.83,9M7.53,9.8L9.08,11.35C9.03,11.56 9,11.77 9,12A3,3 0 0,0 12,15C12.22,15 12.44,14.97 12.65,14.92L14.2,16.47C13.53,16.8 12.79,17 12,17A5,5 0 0,1 7,12C7,11.21 7.2,10.47 7.53,9.8M2,4.27L4.28,6.55L4.73,7C3.08,8.3 1.78,10 1,12C2.73,16.39 7,19.5 12,19.5C13.55,19.5 15.03,19.2 16.38,18.66L16.81,19.08L19.73,22L21,20.73L3.27,3M12,7A5,5 0 0,1 17,12C17,12.64 16.87,13.26 16.64,13.82L19.57,16.75C21.07,15.5 22.27,13.86 23,12C21.27,7.61 17,4.5 12,4.5C10.6,4.5 9.26,4.75 8,5.2L10.17,7.35C10.74,7.13 11.35,7 12,7Z",
      viewType: ViewMode.Hidden,
    },
  ];

  constructor(
    public memory: MemoryService,
    private router: Router,
  ) {}

  get activeItem(): string {
    return this.memory.ActiveSidebarItem.value;
  }

  get sources(): Source[] {
    return Array.from(this.memory.Sources.values());
  }

  navigate(item: SidebarItem) {
    if (this.memory.PlayerState.value === PlayerState.Expanded) {
      this.memory.PlayerState.next(PlayerState.Mini);
    }
    this.memory.ActiveSidebarItem.next(item.id);
    if (item.viewType !== undefined) {
      const event: SidebarNavEvent = {
        viewType: item.viewType,
        mediaTypeFilter: item.mediaTypeFilter,
      };
      this.memory.SidebarNav.next(event);
    }
  }

  openSettings() {
    if (this.memory.PlayerState.value === PlayerState.Expanded) {
      this.memory.PlayerState.next(PlayerState.Mini);
    }
    this.router.navigateByUrl("settings");
  }

  toggleSource(sourceId: number) {
    // Source toggling in sidebar is visual filtering, not enable/disable
    // This filters the current view to only show channels from this source
    const currentItem = [...this.navItems, ...this.browseItems].find(
      (i) => i.id === this.activeItem,
    );
    if (currentItem) {
      const event: SidebarNavEvent = {
        viewType: currentItem.viewType ?? ViewMode.All,
        mediaTypeFilter: currentItem.mediaTypeFilter,
        sourceFilter: [sourceId],
      };
      this.memory.SidebarNav.next(event);
    }
  }

  onCollapse() {
    this.toggleCollapse.emit();
    this.memory.toggleSidebarCollapsed();
  }

  openSearch() {
    this.memory.SearchOverlayOpen.next(true);
  }
}
