import { Component, OnInit, OnDestroy } from "@angular/core";
import { Subscription } from "rxjs";
import { invoke } from "@tauri-apps/api/core";
import { MemoryService } from "../memory.service";
import { Channel } from "../models/channel";
import { ViewMode } from "../models/viewMode";
import { MediaType } from "../models/mediaType";
import { SortType } from "../models/sortType";
import { NodeType } from "../models/nodeType";

interface DashboardRow {
  title: string;
  channels: Channel[];
  loading: boolean;
  groupId?: number;
  loaded?: boolean;
}

@Component({
  selector: "app-dashboard",
  templateUrl: "./dashboard.component.html",
  styleUrl: "./dashboard.component.css",
})
export class DashboardComponent implements OnInit, OnDestroy {
  historyRow: DashboardRow = { title: "Continue Watching", channels: [], loading: true };
  favoritesRow: DashboardRow = { title: "Favorites", channels: [], loading: true };
  categoryRows: DashboardRow[] = [];
  initialLoading = true;
  private observer?: IntersectionObserver;
  private subscriptions: Subscription[] = [];

  readonly viewModeHistory = ViewMode.History;
  readonly viewModeFavorites = ViewMode.Favorites;
  readonly viewModeAll = ViewMode.All;

  constructor(public memory: MemoryService) {}

  ngOnInit() {
    // Try loading immediately if sources are already available
    if (this.memory.Sources.size > 0) {
      this.loadDashboard();
    }
    // Also subscribe to RefreshSources for when sources load later (race condition fix)
    this.subscriptions.push(
      this.memory.RefreshSources.subscribe(() => {
        this.loadDashboard();
      })
    );
  }

  ngOnDestroy() {
    if (this.observer) {
      this.observer.disconnect();
    }
    this.subscriptions.forEach((s) => s.unsubscribe());
  }

  private getBaseFilters() {
    return {
      source_ids: Array.from(this.memory.Sources.keys()),
      media_types: [MediaType.livestream, MediaType.movie, MediaType.serie],
      page: 1,
      use_keywords: false,
      sort: SortType.provider,
    };
  }

  async loadDashboard() {
    const sourceIds = Array.from(this.memory.Sources.keys());
    if (sourceIds.length === 0) {
      this.initialLoading = false;
      this.historyRow.loading = false;
      this.favoritesRow.loading = false;
      return;
    }

    const base = this.getBaseFilters();

    try {
      const [history, favorites, categories] = await Promise.all([
        invoke("search", { filters: { ...base, view_type: ViewMode.History } }).catch(() => []),
        invoke("search", { filters: { ...base, view_type: ViewMode.Favorites } }).catch(() => []),
        invoke("search", { filters: { ...base, view_type: ViewMode.Categories } }).catch(() => []),
      ]);

      this.historyRow.channels = (history as Channel[]).slice(0, 10);
      this.historyRow.loading = false;

      this.favoritesRow.channels = (favorites as Channel[]).slice(0, 10);
      this.favoritesRow.loading = false;

      // Create rows for each category (lazy loaded via IntersectionObserver)
      const cats = categories as Channel[];
      this.categoryRows = cats.slice(0, 6).map((cat) => ({
        title: cat.name || "Unknown",
        channels: [],
        loading: true,
        groupId: cat.id,
        loaded: false,
      }));
    } catch (e) {
      this.historyRow.loading = false;
      this.favoritesRow.loading = false;
    }

    this.initialLoading = false;

    // Setup lazy loading for category rows after DOM renders
    setTimeout(() => this.setupLazyLoading(), 250);
  }

  private setupLazyLoading() {
    this.observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            const index = parseInt(entry.target.getAttribute("data-row-index") || "-1");
            if (index >= 0 && this.categoryRows[index] && !this.categoryRows[index].loaded) {
              this.loadCategoryRow(index);
              this.observer!.unobserve(entry.target);
            }
          }
        });
      },
      { rootMargin: "300px" },
    );

    document.querySelectorAll("[data-row-index]").forEach((el) => {
      this.observer!.observe(el);
    });
  }

  async loadCategoryRow(index: number) {
    const row = this.categoryRows[index];
    if (row.loaded) return;

    const base = this.getBaseFilters();

    try {
      const channels: Channel[] = await invoke("search", {
        filters: {
          ...base,
          view_type: ViewMode.Categories,
          group_id: row.groupId,
        },
      });
      row.channels = channels.slice(0, 10);
    } catch (e) {
      row.channels = [];
    }
    row.loading = false;
    row.loaded = true;
  }

  onSeeAllHistory() {
    this.memory.ActiveSidebarItem.next("history");
    this.memory.SidebarNav.next({ viewType: ViewMode.History });
  }

  onSeeAllFavorites() {
    this.memory.ActiveSidebarItem.next("favorites");
    this.memory.SidebarNav.next({ viewType: ViewMode.Favorites });
  }

  onSeeAllCategory(row: DashboardRow) {
    // Switch to categories view and drill into this group
    this.memory.ActiveSidebarItem.next("categories");
    this.memory.SidebarNav.next({ viewType: ViewMode.Categories });
    // After the categories view loads, drill into this specific group
    setTimeout(() => {
      this.memory.SetNode.next({
        id: row.groupId!,
        name: row.title,
        type: NodeType.Category,
      });
    }, 150);
  }

  hasAnyContent(): boolean {
    return (
      this.historyRow.channels.length > 0 ||
      this.favoritesRow.channels.length > 0 ||
      this.categoryRows.length > 0
    );
  }

  trackByCategoryRow(index: number, row: DashboardRow): string {
    return row.title;
  }

  openSearch() {
    this.memory.SearchOverlayOpen.next(true);
  }
}
