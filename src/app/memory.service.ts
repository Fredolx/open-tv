import { HostListener, Injectable } from "@angular/core";
import { Source } from "./models/source";
import { Subject } from "rxjs";
import { MatMenuTrigger } from "@angular/material/menu";
import { IdName } from "./models/idName";
import { ToastrService } from "ngx-toastr";
import { ErrorService } from "./error.service";
import { Channel } from "./models/channel";
import { NgbModalRef } from "@ng-bootstrap/ng-bootstrap";
import { invoke } from "@tauri-apps/api/core";
import { Settings } from "./models/settings";

@Injectable({
  providedIn: "root",
})
export class MemoryService {
  constructor(
    private toastr: ToastrService,
    private error: ErrorService,
  ) {}
  public SetGroupNode: Subject<IdName> = new Subject();
  public SetSeriesNode: Subject<Channel> = new Subject();
  public Sources: Source[] = [];
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

  async setZoom(zoom?: number) {
    if (zoom  && zoom < 1 || zoom == null) return;
  let settings = await invoke("get_settings") as Settings;
  settings.zoom = zoom;
  await invoke("update_settings", {settings:settings});
  // @ts-ignore
  document.body.style.zoom = settings.zoom
  this.toastr.info("Zoom set to " + settings.zoom.toFixed(2));
  }

  async getZoom() {
    let settings = await invoke("get_settings") as Settings;
    return settings.zoom;
  }
}
