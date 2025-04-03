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
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { RestreamModalComponent } from "../restream-modal/restream-modal.component";

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
  ) {}
  @Input() channel?: Channel;
  @Input() id!: Number;
  @ViewChild(MatMenuTrigger, { static: true }) matMenuTrigger!: MatMenuTrigger;
  menuTopLeftPosition = { x: 0, y: 0 };
  showImage: boolean = true;
  starting: boolean = false;
  // less reactive but prevents the user from seeing the change in action
  alreadyExistsInFav = false;
  mediaTypeEnum = MediaType;
  progress = 0;
  toUnlisten?: UnlistenFn;
  downloading = false;

  ngAfterViewInit(): void {
    if (this.memory.downloadExists(this.channel!.id!)) {
      let download = this.memory.getDownload(this.channel!.id!);
      this.progress = download?.[0]!;
      if (this.progress != 0) this.setDownloadGradient();
      listen<number>("progress", (event) => {
        this.progress = event.payload;
        this.setDownloadGradient();
      }).then((unlisten) => {
        this.downloading = true;
        this.toUnlisten = unlisten;
        download?.[1].subscribe((_) => {
          this.downloading = false;
          if (this.toUnlisten) this.toUnlisten();
          this.clearDownloadGradient();
        });
      });
    }
  }

  setDownloadGradient() {
    let element = this.el.nativeElement.querySelector(`#tile-${this.id}`);
    let background = `linear-gradient(to right, green ${this.progress}%, #343a40 ${this.progress}%)`;
    this.renderer.setStyle(element, "background", background);
  }

  clearDownloadGradient() {
    let element = this.el.nativeElement.querySelector(`#tile-${this.id}`);
    let background = "#343a40";
    this.renderer.setStyle(element, "background", background);
  }

  async click(record = false) {
    if (this.starting === true) return;
    if (this.channel?.media_type == MediaType.group) {
      this.memory.SetGroupNode.next({
        id: this.channel.id!,
        name: this.channel.name!,
      });
      return;
    }
    if (this.channel?.media_type == MediaType.serie) {
      if (!this.memory.SeriesRefreshed.has(this.channel.id!)) {
        this.memory.HideChannels.next(false);
        try {
          await invoke("get_episodes", { channel: this.channel });
          this.memory.SeriesRefreshed.set(this.channel.id!, true);
        } catch (e) {
          this.error.handleError(e, "Failed to fetch series");
        }
      }
      this.memory.SetSeriesNode.next(this.channel);
      return;
    }
    this.starting = true;
    try {
      await invoke("play", { channel: this.channel, record: record });
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
    if (this.channel?.media_type == MediaType.group && !this.isCustom()) return;
    this.alreadyExistsInFav = this.channel!.favorite!;
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

  share() {
    if (this.channel?.media_type == MediaType.group) {
      this.memory.tryIPC(
        `Successfully exported category to Downloads/${this.channel?.id}.otvg`,
        "Failed to export channel",
        () => invoke("share_custom_group", { group: this.channel }),
      );
    } else {
      this.memory.tryIPC(
        `Successfully exported channel to Downloads/${this.channel?.id}.otv`,
        "Failed to export channel",
        () => invoke("share_custom_channel", { channel: this.channel }),
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

  async download() {
    if (this.memory.Loading) return;
    if (this.toUnlisten) this.toUnlisten();
    this.downloading = true;
    this.toUnlisten = await listen<number>("progress", (event) => {
      this.progress = event.payload;
      this.setDownloadGradient();
    });
    this.memory.addDownloadingChannel(this.channel!.id!);
    try {
      await invoke("download", { name: this.channel?.name, url: this.channel?.url });
      this.error.success("Successfully downloaded movie");
    } catch (e) {
      this.error.handleError(e);
    }
    this.memory.notifyDownloadFinished(this.channel!.id!);
    this.memory.removeDownloadingChannel(this.channel!.id!);
    if (this.toUnlisten) this.toUnlisten();
    this.progress = 0;
    this.clearDownloadGradient();
    this.downloading = false;
  }

  ngOnDestroy() {
    if (this.progress != 0) this.memory.setLastDownloadProgress(this.channel!.id!, this.progress);
    if (this.toUnlisten) this.toUnlisten();
  }
}
