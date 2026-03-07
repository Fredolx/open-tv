import { Component, HostListener, OnDestroy, OnInit } from "@angular/core";
import { Subscription } from "rxjs";
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
  playerState: PlayerState = PlayerState.Closed;
  private subscriptions: Subscription[] = [];

  constructor(
    private download: DownloadService,
    public memory: MemoryService,
  ) {}

  ngOnInit() {
    // Subscribe to PlayerState for inline player layout
    this.subscriptions.push(
      this.memory.PlayerState.subscribe((state) => {
        this.playerState = state;
      }),
    );
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

  private isInsideMenuTrigger(element: HTMLElement): boolean {
    return !!element.closest("[mat-menu-trigger-for], [matMenuTriggerFor]");
  }

  showDownloadManager() {
    return this.download.Downloads.size > 0;
  }
}
