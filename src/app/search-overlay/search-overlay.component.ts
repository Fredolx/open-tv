import {
  Component,
  OnInit,
  OnDestroy,
  ViewChild,
  ElementRef,
  HostListener,
} from "@angular/core";
import { trigger, style, transition, animate } from "@angular/animations";
import { Subscription, Subject } from "rxjs";
import { debounceTime, distinctUntilChanged } from "rxjs/operators";
import { invoke } from "@tauri-apps/api/core";
import { MemoryService } from "../memory.service";
import { Channel } from "../models/channel";
import { MediaType } from "../models/mediaType";
import { ViewMode } from "../models/viewMode";
import { SortType } from "../models/sortType";

interface SearchGroup {
  label: string;
  mediaType: MediaType;
  channels: Channel[];
}

@Component({
  selector: "app-search-overlay",
  templateUrl: "./search-overlay.component.html",
  styleUrl: "./search-overlay.component.css",
  animations: [
    trigger("overlayFade", [
      transition(":enter", [
        style({ opacity: 0 }),
        animate("200ms ease-out", style({ opacity: 1 })),
      ]),
      transition(":leave", [
        animate("150ms ease-in", style({ opacity: 0 })),
      ]),
    ]),
    trigger("panelScale", [
      transition(":enter", [
        style({ opacity: 0, transform: "scale(0.96) translateY(-8px)" }),
        animate(
          "250ms cubic-bezier(0.16, 1, 0.3, 1)",
          style({ opacity: 1, transform: "scale(1) translateY(0)" }),
        ),
      ]),
      transition(":leave", [
        animate(
          "150ms ease-in",
          style({ opacity: 0, transform: "scale(0.96) translateY(-8px)" }),
        ),
      ]),
    ]),
  ],
})
export class SearchOverlayComponent implements OnInit, OnDestroy {
  visible = false;
  query = "";
  loading = false;
  groups: SearchGroup[] = [];
  allResults: Channel[] = [];
  totalCount = 0;
  selectedIndex = -1;
  flatResults: Channel[] = [];

  private subscriptions: Subscription[] = [];
  private searchSubject = new Subject<string>();

  @ViewChild("searchInput") searchInput!: ElementRef<HTMLInputElement>;

  constructor(public memory: MemoryService) {}

  ngOnInit() {
    this.subscriptions.push(
      this.memory.SearchOverlayOpen.subscribe((open) => {
        if (open) this.open();
        else this.close();
      }),
    );

    this.subscriptions.push(
      this.searchSubject
        .pipe(debounceTime(250), distinctUntilChanged())
        .subscribe((term) => {
          this.doSearch(term);
        }),
    );
  }

  ngOnDestroy() {
    this.subscriptions.forEach((s) => s.unsubscribe());
  }

  @HostListener("document:keydown.escape")
  onEscape() {
    if (this.visible) this.close();
  }

  open() {
    this.visible = true;
    this.query = "";
    this.groups = [];
    this.allResults = [];
    this.flatResults = [];
    this.totalCount = 0;
    this.selectedIndex = -1;
    this.loading = false;
    setTimeout(() => {
      this.searchInput?.nativeElement?.focus();
    }, 50);
  }

  close() {
    this.visible = false;
  }

  onInput(value: string) {
    this.query = value;
    if (value.trim().length === 0) {
      this.groups = [];
      this.allResults = [];
      this.flatResults = [];
      this.totalCount = 0;
      this.selectedIndex = -1;
      this.loading = false;
      return;
    }
    this.loading = true;
    this.searchSubject.next(value.trim());
  }

  async doSearch(term: string) {
    if (term.length === 0) return;

    const sourceIds = Array.from(this.memory.Sources.keys());
    if (sourceIds.length === 0) {
      this.loading = false;
      return;
    }

    try {
      const results: Channel[] = await invoke("search", {
        filters: {
          source_ids: sourceIds,
          media_types: [
            MediaType.livestream,
            MediaType.movie,
            MediaType.serie,
          ],
          page: 1,
          use_keywords: false,
          sort: SortType.provider,
          view_type: ViewMode.All,
          query: term,
        },
      });

      this.allResults = results;
      this.totalCount = results.length;

      // Group by media type
      const liveChannels = results.filter(
        (c) => c.media_type === MediaType.livestream,
      );
      const movies = results.filter(
        (c) => c.media_type === MediaType.movie,
      );
      const series = results.filter(
        (c) => c.media_type === MediaType.serie,
      );

      this.groups = [];
      if (liveChannels.length > 0) {
        this.groups.push({
          label: "Live TV",
          mediaType: MediaType.livestream,
          channels: liveChannels.slice(0, 5),
        });
      }
      if (movies.length > 0) {
        this.groups.push({
          label: "Movies",
          mediaType: MediaType.movie,
          channels: movies.slice(0, 5),
        });
      }
      if (series.length > 0) {
        this.groups.push({
          label: "Series",
          mediaType: MediaType.serie,
          channels: series.slice(0, 5),
        });
      }

      // Build flat list for keyboard nav
      this.flatResults = [];
      this.groups.forEach((g) => {
        this.flatResults.push(...g.channels);
      });
      this.selectedIndex = this.flatResults.length > 0 ? 0 : -1;
    } catch (e) {
      this.groups = [];
      this.flatResults = [];
      this.totalCount = 0;
    }
    this.loading = false;
  }

  onKeydown(event: KeyboardEvent) {
    if (event.key === "ArrowDown") {
      event.preventDefault();
      if (this.selectedIndex < this.flatResults.length - 1) {
        this.selectedIndex++;
        this.scrollToSelected();
      }
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      if (this.selectedIndex > 0) {
        this.selectedIndex--;
        this.scrollToSelected();
      }
    } else if (event.key === "Enter") {
      event.preventDefault();
      if (
        this.selectedIndex >= 0 &&
        this.selectedIndex < this.flatResults.length
      ) {
        this.selectChannel(this.flatResults[this.selectedIndex]);
      }
    }
  }

  private scrollToSelected() {
    setTimeout(() => {
      const el = document.querySelector(".search-result-item.selected");
      el?.scrollIntoView({ block: "nearest", behavior: "smooth" });
    }, 0);
  }

  selectChannel(channel: Channel) {
    this.close();
    // Open the detail panel for the selected channel
    this.memory.ShowChannelDetail.next(channel);
  }

  getSourceName(channel: Channel): string {
    if (!channel.source_id) return "";
    return this.memory.Sources.get(channel.source_id)?.name || "";
  }

  isSelected(channel: Channel): boolean {
    return (
      this.selectedIndex >= 0 &&
      this.flatResults[this.selectedIndex]?.id === channel.id
    );
  }

  getMediaBadgeClass(type?: MediaType): string {
    switch (type) {
      case MediaType.livestream:
        return "badge-live";
      case MediaType.movie:
        return "badge-movie";
      case MediaType.serie:
        return "badge-series";
      default:
        return "";
    }
  }

  trackByGroup(index: number, group: SearchGroup): string {
    return group.label;
  }

  trackByChannelId(index: number, channel: Channel): number {
    return channel.id!;
  }

  onBackdropClick(event: MouseEvent) {
    if ((event.target as HTMLElement).classList.contains("search-backdrop")) {
      this.close();
    }
  }
}
