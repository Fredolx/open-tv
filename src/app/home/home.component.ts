import {
  AfterViewInit,
  Component,
  ElementRef,
  HostListener,
  OnDestroy,
  ViewChild,
} from "@angular/core";
import { Router } from "@angular/router";
import { AllowIn, ShortcutInput } from "ng-keyboard-shortcuts";
import { Subscription, debounceTime, distinctUntilChanged, fromEvent, map } from "rxjs";
import { MemoryService } from "../memory.service";
import { Channel } from "../models/channel";
import { ViewMode } from "../models/viewMode";
import { MediaType } from "../models/mediaType";
import { ToastrService } from "ngx-toastr";
import { FocusArea, FocusAreaPrefix } from "../models/focusArea";
import { invoke } from "@tauri-apps/api/core";
import { Source } from "../models/source";
import { Filters } from "../models/filters";
import { SourceType } from "../models/sourceType";
import { animate, state, style, transition, trigger } from "@angular/animations";
import { ErrorService } from "../error.service";
import { Settings } from "../models/settings";

@Component({
  selector: "app-home",
  templateUrl: "./home.component.html",
  styleUrl: "./home.component.css",
  animations: [
    trigger("fadeInOut", [
      transition(":enter", [
        style({ opacity: 0, height: 0, padding: "0", margin: "0" }),
        animate("250ms", style({ opacity: 1, height: "*", padding: "*", margin: "*" })),
      ]),
      transition(":leave", [
        style({ opacity: 1, height: "*", padding: "*", margin: "*" }),
        animate("250ms", style({ opacity: 0, height: 0, padding: "0", margin: "0" })),
      ]),
    ]),
    trigger("fade", [
      state(
        "visible",
        style({
          opacity: 1,
        }),
      ),
      state(
        "hidden",
        style({
          opacity: 0,
        }),
      ),
      transition("visible => hidden", [animate("250ms ease-out")]),
      transition("hidden => visible", [animate("250ms ease-in")]),
    ]),
  ],
})
export class HomeComponent implements AfterViewInit, OnDestroy {
  channels: Channel[] = [];
  readonly viewModeEnum = ViewMode;
  readonly mediaTypeEnum = MediaType;
  @ViewChild("search") search!: ElementRef;
  shortcuts: ShortcutInput[] = [];
  focus: number = 0;
  focusArea = FocusArea.Tiles;
  currentWindowSize: number = window.innerWidth;
  subscriptions: Subscription[] = [];
  filters?: Filters;
  chkLiveStream = true;
  chkMovie = true;
  chkSerie = true;
  current_series_name?: string;
  current_group_name?: string;
  reachedMax = false;
  readonly PAGE_SIZE = 36;
  channelsVisible = true;
  prevSearchValue?: String;
  loading = false;
  oldPixelRatio = window.devicePixelRatio;

  constructor(
    private router: Router,
    public memory: MemoryService,
    public toast: ToastrService,
    private error: ErrorService,
  ) {
    this.getSources();
  }

  getSources() {
    let get_settings = invoke("get_settings");
    let get_sources = invoke("get_sources");
    Promise.all([get_settings, get_sources])
      .then((data) => {
        let settings = data[0] as Settings;
        let sources = data[1] as Source[];
        sources
          .filter((x) => x.source_type == SourceType.Custom)
          .map((x) => x.id!)
          .forEach((x) => this.memory.CustomSourceIds?.add(x));
        sources
          .filter((x) => x.source_type == SourceType.Xtream)
          .map((x) => x.id!)
          .forEach((x) => this.memory.XtreamSourceIds.add(x));
        if (this.memory.XtreamSourceIds.size > 0 && !sessionStorage.getItem("epgCheckedOnStart")) {
          sessionStorage.setItem("epgCheckedOnStart", "true");
          invoke("on_start_check_epg");
        }
        this.memory.Sources = sources.filter((x) => x.enabled);
        if (sources.length == 0) this.reset();
        else {
          this.filters = {
            source_ids: this.memory.Sources.map((x) => x.id!),
            view_type: settings.default_view ?? ViewMode.All,
            media_types: [MediaType.livestream, MediaType.movie, MediaType.serie],
            page: 1,
            use_keywords: false,
          };
          this.chkSerie = this.anyXtream();
          if (settings.refresh_on_start === true && !sessionStorage.getItem("refreshedOnStart")) {
            sessionStorage.setItem("refreshedOnStart", "true");
            this.refreshOnStart().then((_) => _);
          }
          this.load().then((_) => _);
         this.memory.setZoom(settings.zoom ??  1.0);
        }
      })
      .catch((e) => {
        this.error.handleError(e);
        this.reset();
      });
  }

  async refreshOnStart() {
    this.toast.info("Refreshing all sources... (refresh on start enabled)");
    await this.memory.tryIPC(
      "Successfully refreshed all sources (refresh on start enabled)",
      "Failed to refresh all sources (refresh on start enabled)",
      async () => {
        await invoke("refresh_all");
      },
    );
  }

  reset() {
    this.router.navigateByUrl("setup");
  }

  async addEvents() {
    this.subscriptions.push(
      this.memory.HideChannels.subscribe((val) => {
        this.channelsVisible = val;
      }),
    );
    this.subscriptions.push(
      this.memory.SetSeriesNode.subscribe(async (channel) => {
        this.clearSearch();
        this.filters!.series_id = parseInt(channel.url!);
        this.filters!.source_ids = [channel.source_id!];
        this.filters!.page = 1;
        this.reachedMax = false;
        this.current_series_name = channel.name;
        await this.load();
      }),
    );
    this.subscriptions.push(
      this.memory.SetGroupNode.subscribe(async (idName) => {
        this.clearSearch();
        this.filters!.group_id = idName.id;
        this.filters!.page = 1;
        this.reachedMax = false;
        this.current_group_name = idName.name;
        await this.load();
      }),
    );
    this.subscriptions.push(
      this.memory.Refresh.subscribe((favs) => {
        if (favs === false || this.filters?.view_type == ViewMode.Favorites) this.load();
      }),
    );
  }

  clearSearch() {
    this.search.nativeElement.value = "";
    this.filters!.query = "";
  }

  async loadMore() {
    this.filters!.page++;
    this.load(true);
  }

  async load(more = false) {
    this.loading = true;
    try {
      let channels: Channel[] = await invoke("search", { filters: this.filters });
      if (!more) {
        this.channels = channels;
        this.channelsVisible = true;
      } else {
        this.channels = this.channels.concat(channels);
      }
      this.reachedMax = channels.length < this.PAGE_SIZE;
    } catch (e) {
      this.error.handleError(e);
    }
    this.loading = false;
  }

  @HostListener("window:scroll", ["$event"])
  async scroll(event: any) {
    if (this.reachedMax === true || this.loading === true) return;
    const scrollHeight = document.documentElement.scrollHeight;
    const scrollTop = window.scrollY || document.documentElement.scrollTop;
    const clientHeight = window.innerHeight || document.documentElement.clientHeight;
    if (scrollTop + clientHeight >= scrollHeight - 1) {
      await this.loadMore();
    }
  }

  @HostListener("window:resize", ["$event"])
  async onResize(event: any) {
    if (this.oldPixelRatio != window.devicePixelRatio) {
    await this.memory.setZoom(window.devicePixelRatio)
    this.oldPixelRatio = window.devicePixelRatio;
    }
  }

  ngAfterViewInit(): void {
    this.addEvents().then((_) => _);
    this.subscriptions.push(
      fromEvent(this.search.nativeElement, "keyup")
        .pipe(
          map((event: any) => {
            if (this.channelsVisible && event.target.value != this.prevSearchValue)
              this.channelsVisible = false;
            this.prevSearchValue = event.target.value;
            return event.target.value;
          }),
          debounceTime(300),
          distinctUntilChanged(),
        )
        .subscribe(async (term: string) => {
          this.focus = 0;
          this.filters!.query = term;
          this.filters!.page = 1;
          this.reachedMax = false;
          await this.load();
        }),
    );

    this.shortcuts.push(
      {
        key: ["ctrl + f", "ctrl + space"],
        label: "Search",
        description: "Go to search",
        preventDefault: true,
        allowIn: [AllowIn.Input],
        command: (_) => this.focusSearch(),
      },
      {
        key: "ctrl + a",
        label: "Switching modes",
        description: "Selects the all channels mode",
        preventDefault: true,
        command: async (_) => await this.switchMode(this.viewModeEnum.All),
      },
      {
        key: "ctrl + s",
        label: "Switching modes",
        description: "Selects the categories channels mode",
        allowIn: [AllowIn.Input],
        command: async (_) => await this.switchMode(this.viewModeEnum.Categories),
      },
      {
        key: "ctrl + d",
        label: "Switching modes",
        description: "Selects the favorites channels mode",
        allowIn: [AllowIn.Input],
        command: async (_) => await this.switchMode(this.viewModeEnum.Favorites),
      },
      {
        key: "ctrl + q",
        label: "Media Type Filters",
        description: "Enable/Disable livestreams",
        preventDefault: true,
        allowIn: [AllowIn.Input],
        command: async (_) => {
          this.chkLiveStream = !this.chkLiveStream;
          this.updateMediaTypes(MediaType.livestream);
        },
      },
      {
        key: "ctrl + w",
        label: "Media Type Filters",
        description: "Enable/Disable movies",
        preventDefault: true,
        allowIn: [AllowIn.Input],
        command: async (_) => {
          this.chkMovie = !this.chkMovie;
          this.updateMediaTypes(MediaType.movie);
        },
      },
      {
        key: "ctrl + e",
        label: "Media Type Filters",
        description: "Enable/Disable series",
        preventDefault: true,
        allowIn: [AllowIn.Input],
        command: async (_) => {
          this.chkSerie = !this.chkSerie;
          this.updateMediaTypes(MediaType.serie);
        },
      },
      {
        key: "left",
        label: "Navigation",
        description: "Go left",
        allowIn: [AllowIn.Input],
        command: async (_) => await this.nav("ArrowLeft"),
      },
      {
        key: "right",
        label: "Navigation",
        description: "Go right",
        allowIn: [AllowIn.Input],
        command: async (_) => await this.nav("ArrowRight"),
      },
      {
        key: "up",
        label: "Navigation",
        description: "Go up",
        allowIn: [AllowIn.Input],
        preventDefault: true,
        command: async (_) => await this.nav("ArrowUp"),
      },
      {
        key: "down",
        label: "Navigation",
        description: "Go down",
        allowIn: [AllowIn.Input],
        preventDefault: true,
        command: async (_) => await this.nav("ArrowDown"),
      },
    );
  }

  updateMediaTypes(mediaType: MediaType) {
    let index = this.filters!.media_types.indexOf(mediaType);
    if (index == -1) this.filters!.media_types.push(mediaType);
    else this.filters!.media_types.splice(index, 1);
    this.filters!.page = 1;
    this.reachedMax = false;
    this.load();
  }

  filtersVisible() {
    return !(
      (this.filters?.view_type == this.viewModeEnum.Categories && !this.filters?.group_id) ||
      this.filters?.series_id
    );
  }

  async switchMode(viewMode: ViewMode) {
    if (viewMode == this.filters?.view_type) return;
    this.filters!.page = 1;
    this.focus = 0;
    this.filters!.series_id = undefined;
    this.filters!.group_id = undefined;
    this.reachedMax = false;
    this.filters!.view_type = viewMode;
    this.clearSearch();
    await this.load();
  }

  searchFocused(): boolean {
    return document.activeElement?.id == "search";
  }

  focusSearch() {
    if (this.searchFocused()) {
      this.selectFirstChannel();
      return;
    }
    this.focus = 0;
    window.scrollTo({ top: 0, behavior: "smooth" });
    this.search.nativeElement.focus({
      preventScroll: true,
    });
  }

  goBackHotkey() {
    if (this.memory.ModalRef) {
      if (
        this.memory.ModalRef.componentInstance.name != "RestreamModalComponent" ||
        !this.memory.ModalRef.componentInstance.started
      )
        this.memory.ModalRef.close("close");
      return;
    }
    if (this.filters?.group_id || this.filters?.series_id) {
      if (this.filters.group_id && this.focusArea == FocusArea.Filters) {
        this.focusArea = FocusArea.Tiles;
        this.focus = 0;
      }
      this.goBack();
    }
    this.closeContextMenu();
    this.selectFirstChannel();
  }

  async goBack() {
    if (this.filters?.series_id) {
      this.filters!.series_id = undefined;
      this.filters.source_ids = this.memory.Sources.map((x) => x.id!);
    } else {
      this.filters!.group_id = undefined;
    }
    this.filters!.page = 1;
    this.clearSearch();
    await this.load();
  }

  openSettings() {
    this.router.navigateByUrl("settings");
  }

  async nav(key: string) {
    let lowSize = this.currentWindowSize < 768;
    if (this.memory.currentContextMenu?.menuOpen || this.memory.ModalRef) {
      return;
    }
    let tmpFocus = 0;
    switch (key) {
      case "ArrowUp":
        tmpFocus -= 3;
        break;
      case "ArrowDown":
        tmpFocus += 3;
        break;
      case "ShiftTab":
      case "ArrowLeft":
        tmpFocus -= 1;
        break;
      case "Tab":
      case "ArrowRight":
        tmpFocus += 1;
        break;
    }
    let goOverSize = this.shortFiltersMode() ? 1 : 2;
    if (lowSize && tmpFocus % 3 == 0 && this.focusArea == FocusArea.Tiles) tmpFocus / 3;
    tmpFocus += this.focus;
    if (tmpFocus < 0) {
      this.changeFocusArea(false);
    } else if (tmpFocus > goOverSize && this.focusArea != FocusArea.Tiles) {
      this.changeFocusArea(true);
    } else if (this.focusArea == FocusArea.Tiles && tmpFocus >= this.filters!.page * 36)
      await this.loadMore();
    else {
      if (tmpFocus >= this.channels.length && this.focusArea == FocusArea.Tiles)
        tmpFocus = (this.channels.length == 0 ? 1 : this.channels.length) - 1;
      this.focus = tmpFocus;
      setTimeout(() => {
        document.getElementById(`${FocusAreaPrefix[this.focusArea]}${this.focus}`)?.focus();
      }, 0);
    }
  }

  shortFiltersMode() {
    return !this.memory.Sources && this.focusArea == FocusArea.Filters;
  }

  anyXtream() {
    return this.memory.Sources.findIndex((x) => x.source_type == SourceType.Xtream) != -1;
  }

  changeFocusArea(down: boolean) {
    let increment = down ? 1 : -1;
    this.focusArea += increment;
    if (this.focusArea == FocusArea.Filters && !this.filtersVisible()) this.focusArea += increment;
    if (this.focusArea < 0) this.focusArea = 0;
    this.applyFocusArea(down);
  }

  applyFocusArea(down: boolean) {
    this.focus = down ? 0 : this.shortFiltersMode() ? 1 : 2;
    let id = FocusAreaPrefix[this.focusArea] + this.focus;
    document.getElementById(id)?.focus();
  }

  //Temporary solution because the ng-keyboard-shortcuts library doesn't seem to support ESC
  @HostListener("document:keydown", ["$event"])
  onKeyDown(event: KeyboardEvent) {
    if (
      event.key == "Escape" ||
      event.key == "BrowserBack" ||
      (event.key == "Backspace" && !this.searchFocused())
    ) {
      this.goBackHotkey();
      event.preventDefault();
    }
    if (event.key == "Tab" && !this.memory.ModalRef) {
      event.preventDefault();
      this.nav(event.shiftKey ? "ShiftTab" : "Tab");
    }
    if (event.key == "Enter" && this.focusArea == FocusArea.Filters)
      (document.activeElement as any).click();
  }

  selectFirstChannel() {
    this.focusArea = FocusArea.Tiles;
    (document.getElementById("first")?.firstChild as HTMLElement)?.focus();
  }

  closeContextMenu() {
    if (this.memory.currentContextMenu?.menuOpen) {
      this.memory.currentContextMenu?.closeMenu();
    }
  }

  ngOnDestroy() {
    this.subscriptions.forEach((x) => x.unsubscribe());
  }

  async toggleKeywords() {
    this.filters!.use_keywords = !this.filters!.use_keywords;
    this.filters!.page = 1;
    this.reachedMax = false;
    await this.load();
  }
}
