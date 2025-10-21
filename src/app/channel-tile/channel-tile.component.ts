import {
  AfterViewInit,
  Component,
  ElementRef,
  Input,
  OnDestroy,
  Renderer2,
  ViewChild,
} from "@angular/core";
import { MatMenuTrigger } from "@angular/material/menu";
import { Channel } from "../models/channel";
import { MemoryService } from "../memory.service";
import { MediaType } from "../models/mediaType";
import { invoke } from "@tauri-apps/api/core";
import { ToastrService } from "ngx-toastr";
import { ErrorService } from "../error.service";
import { NgbModal } from "@ng-bootstrap/ng-bootstrap";
import { EditChannelModalComponent } from "../edit-channel-modal/edit-channel-modal.component";
import { EditGroupModalComponent } from "../edit-group-modal/edit-group-modal.component";
import { DeleteGroupModalComponent } from "../delete-group-modal/delete-group-modal.component";
import { EpgModalComponent } from "../epg-modal/epg-modal.component";
import { EPG } from "../models/epg";
import { RestreamModalComponent } from "../restream-modal/restream-modal.component";
import { DownloadService } from "../download.service";
import { Download } from "../models/download";
import { Subscription, take } from "rxjs";
import { save } from "@tauri-apps/plugin-dialog";
import { CHANNEL_EXTENSION, GROUP_EXTENSION, RECORD_EXTENSION } from "../models/extensions";
import { getDateFormatted, getExtension, sanitizeFileName } from "../utils";
import { NodeType, fromMediaType } from "../models/nodeType";

@Component({
  selector: "app-channel-tile",
  templateUrl: "./channel-tile.component.html",
  styleUrl: "./channel-tile.component.css",
})
export class ChannelTileComponent implements OnDestroy, AfterViewInit {
  constructor(
    public memory: MemoryService,
    private toastr: ToastrService,
    private error: ErrorService,
    private modal: NgbModal,
    private el: ElementRef,
    private renderer: Renderer2,
    private download: DownloadService,
  ) {}
  @Input() channel?: Channel;
  @Input() id!: number;
  @ViewChild(MatMenuTrigger, { static: true }) matMenuTrigger!: MatMenuTrigger;
  menuTopLeftPosition = { x: 0, y: 0 };
  showImage: boolean = true;
  starting: boolean = false;
  alreadyExistsInFav = false;
  downloading = false;
  mediaTypeEnum = MediaType;
  subscriptions: Subscription[] = [];

  ngAfterViewInit(): void {
    this.getExistingDownload();
  }

  setDownloadGradient(progress: number) {
    let element = this.el.nativeElement.querySelector(`#tile-${this.id}`);
    let background = `linear-gradient(to right, green ${progress}%, #343a40 ${progress}%)`;
    this.renderer.setStyle(element, "background", background);
  }

  clearDownloadGradient() {
    let element = this.el.nativeElement.querySelector(`#tile-${this.id}`);
    let background = "#343a40";
    this.renderer.setStyle(element, "background", background);
  }

  async click(record = false) {
    if (this.starting === true) {
      try {
        await invoke("cancel_play", { sourceId: this.channel?.source_id, channelId: this.channel?.id });
      } catch (e) {
        this.error.handleError(e);
      }
      return;
    }
    if (
      this.channel?.media_type == MediaType.serie ||
      this.channel?.media_type == MediaType.group ||
      this.channel?.media_type == MediaType.season
    ) {
      if (
        this.channel.media_type == MediaType.serie &&
        !this.memory.SeriesRefreshed.has(this.channel.id!)
      ) {
        this.memory.HideChannels.next(false);
        try {
          await invoke("get_episodes", { channel: this.channel });
          this.memory.SeriesRefreshed.set(this.channel.id!, true);
        } catch (e) {
          this.error.handleError(e, "Failed to fetch series");
        }
      }
      this.memory.SetNode.next({
        id:
          this.channel?.media_type == MediaType.serie
            ? parseInt(this.channel.url!)
            : this.channel.id!,
        name: this.channel.name!,
        type: fromMediaType(this.channel.media_type),
        sourceId: this.channel.source_id,
      });
      return;
    }
    let file = undefined;
    if (record && (this.memory.IsContainer || this.memory.AlwaysAskSave)) {
      file = await save({
        canCreateDirectories: true,
        title: "Select where to save recording",
        defaultPath: `${sanitizeFileName(this.channel?.name!)}_${getDateFormatted()}${RECORD_EXTENSION}`,
      });
      if (!file) return;
    }
    this.starting = true;
    this.memory.SetFocus.next(this.id);
    try {
      await invoke("play", { channel: this.channel, record: record, recordPath: file });
    } catch (e) {
      this.error.handleError(e);
    }
    invoke("add_last_watched", { id: this.channel?.id }).catch((e) => {
      console.error(e);
      this.error.handleError(e);
    });
    this.starting = false;
  }

  onRightClick(event: MouseEvent) {
    if (
      (this.channel?.media_type == MediaType.group && !this.isCustom()) ||
      this.channel?.media_type == MediaType.season
    )
      return;
    this.alreadyExistsInFav = this.channel!.favorite!;
    this.downloading = this.isDownloading();
    event.preventDefault();
    this.menuTopLeftPosition.x = event.clientX;
    this.menuTopLeftPosition.y = event.clientY;
    if (this.memory.currentContextMenu?.menuOpen) this.memory.currentContextMenu.closeMenu();
    this.memory.currentContextMenu = this.matMenuTrigger;
    this.matMenuTrigger.openMenu();
  }

  onError(event: Event) {
    this.showImage = false;
  }

  async favorite() {
    let call = "favorite_channel";
    let msg = `Added ${this.channel?.name} to favorites`;
    if (this.channel!.favorite) {
      call = "unfavorite_channel";
      msg = `Removed ${this.channel?.name} from favorites`;
    }
    try {
      await invoke(call, { channelId: this.channel!.id });
      if (this.channel!.favorite) this.memory.Refresh.next(true);
      this.channel!.favorite = !this.channel!.favorite;
      this.toastr.success(msg);
    } catch (e) {
      this.error.handleError(e, `Failed to add/remove ${this.channel?.name} to/from favorites`);
    }
  }

  async record() {
    await this.click(true);
  }

  isMovie() {
    return this.channel?.media_type == MediaType.movie;
  }

  isLivestream() {
    return this.channel?.media_type == MediaType.livestream;
  }

  isCustom(): boolean {
    return this.memory.CustomSourceIds!.has(this.channel?.source_id!);
  }

  showEPG(): boolean {
    return (
      this.channel?.media_type == MediaType.livestream &&
      !this.isCustom() &&
      this.memory.XtreamSourceIds.has(this.channel.source_id!)
    );
  }

  async showEPGModal() {
    try {
      let data: EPG[] = await invoke("get_epg", { channel: this.channel });
      if (data.length == 0) {
        this.toastr.info("No EPG data for this channel");
        return;
      }
      this.memory.ModalRef = this.modal.open(EpgModalComponent, {
        backdrop: "static",
        size: "xl",
        keyboard: false,
      });
      this.memory.ModalRef.result.then((_) => (this.memory.ModalRef = undefined));
      this.memory.ModalRef.componentInstance.epg = data;
      this.memory.ModalRef.componentInstance.name = this.channel?.name;
      this.memory.ModalRef.componentInstance.channelId = this.channel?.id;
    } catch (e) {
      this.error.handleError(
        e,
        "Missing stream id. Please refresh your sources (Settings -> Refresh All) to enable the EPG feature",
      );
    }
  }

  edit() {
    if (this.channel?.media_type == MediaType.group) this.edit_group();
    else {
      this.edit_channel();
    }
  }

  edit_group() {
    this.memory.ModalRef = this.modal.open(EditGroupModalComponent, {
      backdrop: "static",
      size: "xl",
      keyboard: false,
    });
    this.memory.ModalRef.result.then((_) => (this.memory.ModalRef = undefined));
    this.memory.ModalRef.componentInstance.name = "EditCustomGroupModal";
    this.memory.ModalRef.componentInstance.editing = true;
    this.memory.ModalRef.componentInstance.group = {
      id: this.channel!.id,
      name: this.channel!.name,
      image: this.channel!.image,
      source_id: this.channel!.source_id,
    };
    this.memory.ModalRef.componentInstance.originalName = this.channel!.name;
  }

  edit_channel() {
    this.memory.ModalRef = this.modal.open(EditChannelModalComponent, {
      backdrop: "static",
      size: "xl",
      keyboard: false,
    });
    this.memory.ModalRef.result.then((_) => (this.memory.ModalRef = undefined));
    this.memory.ModalRef.componentInstance.name = "EditCustomChannelModal";
    this.memory.ModalRef.componentInstance.editing = true;
    this.memory.ModalRef.componentInstance.channel.data = { ...this.channel };
  }

  async share() {
    let entityName = this.channel?.media_type == MediaType.group ? "group" : "channel";
    let file = await save({
      canCreateDirectories: true,
      title: `Select where to export ${entityName}`,
      defaultPath:
        sanitizeFileName(this.channel?.name!) +
        (this.channel?.media_type == MediaType.group ? GROUP_EXTENSION : CHANNEL_EXTENSION),
    });
    if (!file) {
      return;
    }
    if (this.channel?.media_type == MediaType.group) {
      this.memory.tryIPC(
        `Successfully exported category to ${file}`,
        "Failed to export channel",
        () => invoke("share_custom_group", { group: this.channel, path: file }),
      );
    } else {
      this.memory.tryIPC(
        `Successfully exported channel to ${file}`,
        "Failed to export channel",
        () => invoke("share_custom_channel", { channel: this.channel, path: file }),
      );
    }
  }

  async delete() {
    if (this.channel?.media_type == MediaType.group) this.deleteGroup();
    else await this.deleteChannel();
  }

  async deleteGroup() {
    try {
      if (await invoke("group_not_empty", { id: this.channel?.id })) {
        this.openDeleteGroupModal();
      } else await this.deleteGroupNoReplace();
    } catch (e) {
      this.error.handleError(e);
    }
  }

  async deleteGroupNoReplace() {
    try {
      await invoke("delete_custom_group", {
        id: this.channel?.id,
        doChannelsUpdate: false,
      });
      this.memory.Refresh.next(false);
      this.error.success("Successfully deleted category");
    } catch (e) {
      this.error.handleError(e);
    }
  }

  openDeleteGroupModal() {
    this.memory.ModalRef = this.modal.open(DeleteGroupModalComponent, {
      backdrop: "static",
      size: "xl",
      keyboard: false,
    });
    this.memory.ModalRef.result.then((_) => (this.memory.ModalRef = undefined));
    this.memory.ModalRef.componentInstance.name = "DeleteGroupModal";
    this.memory.ModalRef.componentInstance.group = { ...this.channel };
  }

  openRestreamModal() {
    this.memory.ModalRef = this.modal.open(RestreamModalComponent, {
      backdrop: "static",
      size: "xl",
      keyboard: false,
    });
    this.memory.ModalRef.componentInstance.channel = this.channel;
    this.memory.ModalRef.componentInstance.name = "RestreamModalComponent";
    this.memory.ModalRef.result.then((_) => (this.memory.ModalRef = undefined));
  }

  async deleteChannel() {
    await this.memory.tryIPC("Successfully deleted channel", "Failed to delete channel", () =>
      invoke("delete_custom_channel", { id: this.channel?.id }),
    );
    this.memory.Refresh.next(false);
  }

  isDownloading() {
    return this.download.Downloads.has(this.channel!.id!.toString());
  }

  async downloadVod() {
    let file = undefined;
    if (this.memory.IsContainer || this.memory.AlwaysAskSave) {
      file = await save({
        canCreateDirectories: true,
        title: "Select where to download movie",
        defaultPath: `${sanitizeFileName(this.channel?.name!)}.${getExtension(this.channel?.url!)}`,
      });
      if (!file) {
        return;
      }
    }
    let download = await this.download.addDownload(
      this.channel!.id!.toString(),
      this.channel!.name!,
      this.channel?.url!,
    );
    this.downloadSubscribe(download);
    await this.download.download(download.id, file);
  }

  async cancelDownload() {
    await this.download.abortDownload(this.channel!.id!.toString());
  }

  getExistingDownload() {
    let download = this.download.Downloads.get(this.channel!.id!.toString());
    if (download) {
      this.setDownloadGradient(download.progress);
      this.downloadSubscribe(download);
    }
  }

  downloadSubscribe(download: Download) {
    let progressUpdate = download.progressUpdate.subscribe((progress) => {
      this.setDownloadGradient(progress);
      if (progress == 100) progressUpdate.unsubscribe();
    });
    this.subscriptions.push(progressUpdate);
    this.subscriptions.push(
      download.complete.pipe(take(1)).subscribe((_) => {
        progressUpdate.unsubscribe();
        this.clearDownloadGradient();
      }),
    );
  }

  ngOnDestroy() {
    this.subscriptions.forEach((x) => x.unsubscribe());
  }
}
