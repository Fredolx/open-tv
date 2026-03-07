import { Injectable } from "@angular/core";
import { Source } from "./models/source";
import { BehaviorSubject, Subject } from "rxjs";
import { MatMenuTrigger } from "@angular/material/menu";
import { ToastrService } from "ngx-toastr";
import { ErrorService } from "./error.service";
import { NgbModalRef } from "@ng-bootstrap/ng-bootstrap";
import { invoke } from "@tauri-apps/api/core";
import { SortType } from "./models/sortType";
import { LAST_SEEN_VERSION } from "./models/localStorage";
import { SetNodeDTO } from "./models/setNodeDTO";
import { SidebarNavEvent } from "./models/sidebarNavEvent";
import { ViewDensity } from "./models/viewDensity";
import { Channel } from "./models/channel";
import { PlayerEngine } from "./models/playerEngine";
import { PlayerState } from "./models/playerState";
import { Settings } from "./models/settings";

@Injectable({
  providedIn: "root",
})
export class MemoryService {
  constructor(
    private toastr: ToastrService,
    private error: ErrorService,
  ) {
    invoke("is_container").then((val) => (this.IsContainer = val as boolean));
    invoke("get_settings").then((s) => {
      const settings = s as Settings;
      this.PlayerEngine.next(settings.player_engine ?? PlayerEngine.Web);
      this.AlwaysAskSave = settings.always_ask_save;
    });

    // Restore persisted UI preferences
    const savedDensity = localStorage.getItem("ftv-view-density");
    if (savedDensity !== null) {
      this.ViewDensity.next(parseInt(savedDensity) as ViewDensity);
    }
    const savedCollapsed = localStorage.getItem("ftv-sidebar-collapsed");
    if (savedCollapsed !== null) {
      this.SidebarCollapsed.next(savedCollapsed === "true");
    }
  }

  // Existing state
  public SetNode: Subject<SetNodeDTO> = new Subject();
  public SetFocus: Subject<number> = new Subject();
  public Sort: BehaviorSubject<[number, boolean]> = new BehaviorSubject<[number, boolean]>([
    SortType.provider,
    false,
  ]);
  public Sources: Map<number, Source> = new Map();
  public currentContextMenu?: MatMenuTrigger;
  public Loading = false;
  public Refresh: Subject<boolean> = new Subject();
  public RefreshSources: Subject<boolean> = new Subject();
  public AddingAdditionalSource = false;
  public SeriesRefreshed: Map<number, boolean> = new Map();
  public HideChannels: Subject<boolean> = new Subject();
  public CustomSourceIds: Set<number> = new Set();
  public XtreamSourceIds: Set<number> = new Set();
  public ModalRef?: NgbModalRef;
  public Watched_epgs: Set<string> = new Set();
  private downloadingChannels: Map<number, [number, Subject<boolean>]> = new Map();
  public LoadingNotification: boolean = false;
  public AppVersion?: string;
  public trayEnabled?: boolean;
  public IsContainer?: boolean;
  public AlwaysAskSave?: boolean;

  // Sidebar state
  public SidebarNav: Subject<SidebarNavEvent> = new Subject();
  public ActiveSidebarItem: BehaviorSubject<string> = new BehaviorSubject<string>("home");
  public SidebarCollapsed: BehaviorSubject<boolean> = new BehaviorSubject<boolean>(false);

  // View density state
  public ViewDensity: BehaviorSubject<ViewDensity> = new BehaviorSubject<ViewDensity>(ViewDensity.GridLarge);

  // Channel detail panel
  public ShowChannelDetail: Subject<Channel> = new Subject();

  // Now playing state
  public NowPlaying: BehaviorSubject<Channel | null> = new BehaviorSubject<Channel | null>(null);
  // Inline player state
  public PlayerState: BehaviorSubject<PlayerState> = new BehaviorSubject<PlayerState>(PlayerState.Closed);
  public PlayerEngine: BehaviorSubject<number> = new BehaviorSubject<number>(0);
  public LocalProxyRunning: boolean = false;

  // Search overlay
  public SearchOverlayOpen: Subject<boolean> = new Subject();

  toggleSidebarCollapsed() {
    const newVal = !this.SidebarCollapsed.value;
    this.SidebarCollapsed.next(newVal);
    localStorage.setItem("ftv-sidebar-collapsed", String(newVal));
  }

  setViewDensity(density: ViewDensity) {
    this.ViewDensity.next(density);
    localStorage.setItem("ftv-view-density", String(density));
  }

  async tryIPC<T>(
    successMessage: string,
    errorMessage: string,
    action: () => Promise<T>,
  ): Promise<boolean> {
    this.Loading = true;
    let error = false;
    try {
      await action();
      this.toastr.success(successMessage);
    } catch (e) {
      this.error.handleError(e, errorMessage);
      error = true;
    }
    this.Loading = false;
    return error;
  }

  async get_epg_ids() {
    let data = await invoke("get_epg_ids");
    let set = new Set(data as Array<string>);
    this.Watched_epgs = set;
  }

  addDownloadingChannel(id: number) {
    this.downloadingChannels.set(id, [0, new Subject()]);
  }

  notifyDownloadFinished(id: number) {
    this.downloadingChannels.get(id)?.[1].next(true);
  }

  removeDownloadingChannel(id: number) {
    this.downloadingChannels.delete(id);
  }

  downloadExists(id: number) {
    return this.downloadingChannels.has(id);
  }

  getDownload(id: number) {
    return this.downloadingChannels.get(id);
  }

  setLastDownloadProgress(id: number, progress: number) {
    this.downloadingChannels.get(id)![0] = progress;
  }

  updateVersion() {
    if (this.AppVersion) localStorage.setItem(LAST_SEEN_VERSION, this.AppVersion);
  }
}
