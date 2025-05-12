import { Component, ElementRef, HostListener, ViewChild } from "@angular/core";
import { debounceTime, distinctUntilChanged, fromEvent, map, Subscription } from "rxjs";
import { Settings } from "../models/settings";
import { invoke } from "@tauri-apps/api/core";
import { Router } from "@angular/router";
import { open } from "@tauri-apps/plugin-dialog";
import { Source } from "../models/source";
import { MemoryService } from "../memory.service";
import { ViewMode } from "../models/viewMode";
import { NgbModal } from "@ng-bootstrap/ng-bootstrap";
import { ConfirmDeleteModalComponent } from "../confirm-delete-modal/confirm-delete-modal.component";
import { SORT_TYPES, SortType, getSortTypeText } from "../models/sortType";

@Component({
  selector: "app-settings",
  templateUrl: "./settings.component.html",
  styleUrl: "./settings.component.css",
})
export class SettingsComponent {
  subscriptions: Subscription[] = [];
  settings: Settings = {
    use_stream_caching: true,
    default_view: ViewMode.All,
    volume: 100,
    restream_port: 3000,
    enable_tray_icon: true,
    zoom: 100,
    default_sort: SortType.provider,
    enable_hwdec: true,
    always_ask_save: false,
    enable_gpu: false,
  };
  viewModeEnum = ViewMode;
  sources: Source[] = [];
  sortTypes = SORT_TYPES;
  @ViewChild("mpvParams") mpvParams!: ElementRef;

  constructor(
    private router: Router,
    public memory: MemoryService,
    private nav: Router,
    private modal: NgbModal,
  ) {}

  _getSortTypeText(sortType: SortType) {
    return getSortTypeText(sortType);
  }

  isInputFocused(): boolean {
    const activeElement = document.activeElement;
    return (
      activeElement instanceof HTMLInputElement ||
      activeElement instanceof HTMLTextAreaElement ||
      activeElement instanceof HTMLSelectElement
    );
  }

  @HostListener("document:keydown", ["$event"])
  onKeyDown(event: KeyboardEvent) {
    if (
      event.key == "Escape" ||
      event.key == "BrowserBack" ||
      (event.key == "Backspace" && !this.isInputFocused())
    ) {
      if (this.memory.ModalRef) {
        this.memory.ModalRef.close("close");
      } else {
        this.goBack();
      }
      event.preventDefault();
    }
  }

  ngOnInit(): void {
    this.getSettings();
    this.getSources();
  }

  getSettings() {
    invoke("get_settings").then((x) => {
      this.settings = x as Settings;
      if (this.settings.use_stream_caching == undefined) this.settings.use_stream_caching = true;
      if (this.settings.default_view == undefined) this.settings.default_view = ViewMode.All;
      if (this.settings.volume == undefined) this.settings.volume = 100;
      if (this.settings.restream_port == undefined) this.settings.restream_port = 3000;
      if (this.settings.enable_tray_icon == undefined) this.settings.enable_tray_icon = true;
      if (this.settings.zoom == undefined) this.settings.zoom = 100;
      if (this.settings.default_sort == undefined) this.settings.default_sort = SortType.provider;
      if (this.settings.enable_hwdec == undefined) this.settings.enable_hwdec = true;
      if (this.settings.always_ask_save == undefined) this.settings.always_ask_save = false;
      if (this.settings.enable_gpu == undefined) this.settings.enable_gpu = false;
    });
  }

  getSources() {
    invoke("get_sources").then((x) => {
      this.sources = x as Source[];
      if (this.sources.length == 0) {
        this.memory.AddingAdditionalSource = false;
        this.nav.navigateByUrl("setup");
      }
    });
  }

  ngAfterViewInit(): void {
    this.subscriptions.push(
      fromEvent(this.mpvParams.nativeElement, "keyup")
        .pipe(
          map((event: any) => {
            return event.target.value;
          }),
          debounceTime(500),
          distinctUntilChanged(),
        )
        .subscribe(async () => {
          await this.updateSettings();
        }),
    );
    this.subscriptions.push(
      this.memory.RefreshSources.subscribe((_) => {
        this.getSources();
      }),
    );
  }

  addSource() {
    this.memory.AddingAdditionalSource = true;
    this.nav.navigateByUrl("setup");
  }

  async refreshAll() {
    this.memory.SeriesRefreshed.clear();
    await this.memory.tryIPC("Successfully updated all sources", "Failed to refresh sources", () =>
      invoke("refresh_all"),
    );
  }

  async goBack() {
    await this.updateSettings();
    this.router.navigateByUrl("");
  }

  async updateSettings() {
    if (this.settings.mpv_params) this.settings.mpv_params = this.settings.mpv_params?.trim();
    await invoke("update_settings", { settings: this.settings });
  }

  async selectFolder() {
    const folder = await open({
      multiple: false,
      directory: true,
      canCreateDirectories: true,
    });
    if (folder) {
      this.settings.recording_path = folder;
      await this.updateSettings();
    }
  }

  async nuke() {
    this.memory.ModalRef = this.modal.open(ConfirmDeleteModalComponent, {
      backdrop: "static",
      size: "xl",
      keyboard: false,
    });
    this.memory.ModalRef.result.then((_) => (this.memory.ModalRef = undefined));
    this.memory.ModalRef.componentInstance.name = "ConfirmDeleteModal";
  }

  async clearHistory() {
    await this.memory.tryIPC(
      "History cleared successfully",
      "Failed to clear history",
      async () => {
        await invoke("clear_history");
      },
    );
  }

  ngOnDestroy(): void {
    this.subscriptions.forEach((x) => x.unsubscribe());
  }
}
