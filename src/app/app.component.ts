import { Component, HostListener, OnDestroy, OnInit } from "@angular/core";
import { NavigationEnd, Router } from "@angular/router";
import { Subscription, filter } from "rxjs";
import { DownloadService } from "./download.service";
import { MemoryService } from "./memory.service";
import { PlayerState } from "./models/playerState";

@Component({
  selector: "app-root",
  templateUrl: "./app.component.html",
  styleUrl: "./app.component.css",
})
export class AppComponent implements OnInit, OnDestroy {
  title = "open-tv";
  isHomeRoute = false;
  sidebarCollapsed = false;
  hasNowPlaying = false;
  playerState: PlayerState = PlayerState.Closed;
  private subscriptions: Subscription[] = [];

  constructor(
    private download: DownloadService,
    private router: Router,
    public memory: MemoryService,
  ) {}

  ngOnInit() {
    // Track current route for sidebar visibility
    this.subscriptions.push(
      this.router.events
        .pipe(filter((event) => event instanceof NavigationEnd))
        .subscribe((event) => {
          const navEnd = event as NavigationEnd;
          this.isHomeRoute = navEnd.urlAfterRedirects === "/" || navEnd.urlAfterRedirects === "";
        }),
    );

    // Subscribe to sidebar collapsed state
    this.subscriptions.push(
      this.memory.SidebarCollapsed.subscribe((collapsed) => {
        this.sidebarCollapsed = collapsed;
      }),
    );

    // Subscribe to NowPlaying for bottom padding
    this.subscriptions.push(
      this.memory.NowPlaying.subscribe((ch) => {
        this.hasNowPlaying = ch !== null;
      }),
    );

    // Subscribe to PlayerState for inline player layout
    this.subscriptions.push(
      this.memory.PlayerState.subscribe((state) => {
        this.playerState = state;
      }),
    );

    // Check initial route
    this.isHomeRoute = this.router.url === "/" || this.router.url === "";
  }

  ngOnDestroy() {
    this.subscriptions.forEach((s) => s.unsubscribe());
  }

  @HostListener("document:contextmenu", ["$event"])
  onRightClick(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (this.isInsideMenuTrigger(target)) {
      return;
    }
    event.preventDefault();
  }

  @HostListener("document:keydown", ["$event"])
  onGlobalKeydown(event: KeyboardEvent) {
    // Ctrl+K / Cmd+K for search overlay
    if ((event.ctrlKey || event.metaKey) && event.key === "k") {
      event.preventDefault();
      this.memory.SearchOverlayOpen.next(true);
    }
  }

  private isInsideMenuTrigger(element: HTMLElement): boolean {
    return !!element.closest("[mat-menu-trigger-for], [matMenuTriggerFor]");
  }

  showDownloadManager() {
    return this.download.Downloads.size > 0;
  }

  onSidebarToggle() {
    this.memory.toggleSidebarCollapsed();
  }
}
