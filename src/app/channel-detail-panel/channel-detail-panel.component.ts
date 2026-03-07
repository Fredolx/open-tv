import { Component, OnInit, OnDestroy, HostListener } from "@angular/core";
import { trigger, style, transition, animate } from "@angular/animations";
import { Subscription } from "rxjs";
import { invoke } from "@tauri-apps/api/core";
import { MemoryService } from "../memory.service";
import { Channel } from "../models/channel";
import { MediaType } from "../models/mediaType";
import { EPG } from "../models/epg";
import { ErrorService } from "../error.service";
import { PlayerEngine } from "../models/playerEngine";
import { PlayerState } from "../models/playerState";
import { ToastrService } from "ngx-toastr";
import { NgbModal } from "@ng-bootstrap/ng-bootstrap";
import { EpgModalComponent } from "../epg-modal/epg-modal.component";
import { RestreamModalComponent } from "../restream-modal/restream-modal.component";
import { RecordingService } from "../recording.service";

@Component({
  selector: "app-channel-detail-panel",
  templateUrl: "./channel-detail-panel.component.html",
  styleUrl: "./channel-detail-panel.component.css",
  animations: [
    trigger("slideUp", [
      transition(":enter", [
        style({ transform: "translateY(100%)", opacity: 0 }),
        animate(
          "400ms cubic-bezier(0.16, 1, 0.3, 1)",
          style({ transform: "translateY(0)", opacity: 1 }),
        ),
      ]),
      transition(":leave", [
        animate(
          "250ms ease-in",
          style({ transform: "translateY(100%)", opacity: 0 }),
        ),
      ]),
    ]),
    trigger("fadeBackdrop", [
      transition(":enter", [
        style({ opacity: 0 }),
        animate("300ms ease-out", style({ opacity: 1 })),
      ]),
      transition(":leave", [
        animate("200ms ease-in", style({ opacity: 0 })),
      ]),
    ]),
  ],
})
export class ChannelDetailPanelComponent implements OnInit, OnDestroy {
  visible = false;
  channel?: Channel;
  nowPlaying?: EPG;
  upNext?: EPG;
  epgProgress = 0;
  epgLoading = false;
  playing = false;

  private subscription?: Subscription;
  private progressInterval?: ReturnType<typeof setInterval>;

  constructor(
    public memory: MemoryService,
    private toastr: ToastrService,
    private error: ErrorService,
    private modal: NgbModal,
    public recording: RecordingService,
  ) {}

  ngOnInit() {
    this.subscription = this.memory.ShowChannelDetail.subscribe((channel) => {
      this.open(channel);
    });
  }

  ngOnDestroy() {
    this.subscription?.unsubscribe();
    this.clearProgressInterval();
  }

  @HostListener("document:keydown.escape")
  onEscape() {
    if (this.visible) this.close();
  }

  open(channel: Channel) {
    this.channel = channel;
    this.nowPlaying = undefined;
    this.upNext = undefined;
    this.epgProgress = 0;
    this.epgLoading = false;
    this.playing = false;
    this.visible = true;

    if (this.isLive() && this.hasEPG()) {
      this.loadEPG();
    }
  }

  close() {
    this.visible = false;
    this.clearProgressInterval();
  }

  isLive(): boolean {
    return this.channel?.media_type === MediaType.livestream;
  }

  isMovie(): boolean {
    return this.channel?.media_type === MediaType.movie;
  }

  hasEPG(): boolean {
    return (
      this.channel?.media_type === MediaType.livestream &&
      !this.memory.CustomSourceIds.has(this.channel.source_id!) &&
      this.memory.XtreamSourceIds.has(this.channel.source_id!)
    );
  }

  getSourceName(): string {
    if (!this.channel?.source_id) return "";
    return this.memory.Sources.get(this.channel.source_id)?.name || "";
  }

  getMediaLabel(): string {
    switch (this.channel?.media_type) {
      case MediaType.livestream:
        return "LIVE";
      case MediaType.movie:
        return "MOVIE";
      case MediaType.serie:
        return "SERIES";
      default:
        return "";
    }
  }

  getBadgeClass(): string {
    switch (this.channel?.media_type) {
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

  async loadEPG() {
    if (!this.channel) return;
    this.epgLoading = true;
    try {
      const data: EPG[] = await invoke("get_epg", { channel: this.channel });
      const nowIdx = data.findIndex((e) => e.now_playing);
      if (nowIdx >= 0) {
        this.nowPlaying = data[nowIdx];
        if (nowIdx + 1 < data.length) {
          this.upNext = data[nowIdx + 1];
        }
        this.updateProgress();
        this.progressInterval = setInterval(() => this.updateProgress(), 30000);
      }
    } catch (e) {
      // EPG not available for this channel
    }
    this.epgLoading = false;
  }

  updateProgress() {
    if (!this.nowPlaying) return;
    const now = Date.now() / 1000;
    const start = this.nowPlaying.start_timestamp;
    let end: number;
    if (this.upNext) {
      end = this.upNext.start_timestamp;
    } else {
      end = start + 3600;
    }
    const total = end - start;
    if (total <= 0) {
      this.epgProgress = 0;
      return;
    }
    this.epgProgress = Math.min(
      100,
      Math.max(0, ((now - start) / total) * 100),
    );
  }

  private clearProgressInterval() {
    if (this.progressInterval) {
      clearInterval(this.progressInterval);
      this.progressInterval = undefined;
    }
  }

  async play() {
    if (!this.channel || this.playing) return;
    const engine = this.memory.PlayerEngine.value ?? PlayerEngine.Web;

    // Web / EmbeddedMpv: use inline player
    if (engine === PlayerEngine.Web || engine === PlayerEngine.EmbeddedMpv) {
      this.memory.NowPlaying.next(this.channel);
      this.memory.PlayerState.next(PlayerState.Expanded);
      invoke("add_last_watched", { id: this.channel.id }).catch(console.error);
      this.close();
      return;
    }

    // External MPV: playback only
    this.playing = true;
    this.memory.NowPlaying.next(this.channel);
    this.close();
    try {
      await invoke("play", {
        channel: this.channel,
        record: false,
        recordPath: null,
      });
      invoke("add_last_watched", { id: this.channel.id }).catch(
        console.error,
      );
    } catch (e) {
      this.error.handleError(e);
    }
    this.playing = false;
    if (this.memory.NowPlaying.value?.id === this.channel?.id) {
      this.memory.NowPlaying.next(null);
    }
  }

  async doRecord() {
    if (!this.channel) return;
    await this.recording.toggleRecording(this.channel);
  }

  isChannelRecording(): boolean {
    return !!this.channel?.id && this.recording.isRecording(this.channel.id);
  }

  isRecordingBusy(): boolean {
    return !!this.channel?.id && this.recording.isBusy(this.channel.id);
  }

  async toggleFavorite() {
    if (!this.channel) return;
    const wasFav = this.channel.favorite;
    const call = wasFav ? "unfavorite_channel" : "favorite_channel";
    const msg = wasFav
      ? `Removed "${this.channel.name}" from favorites`
      : `Added "${this.channel.name}" to favorites`;
    try {
      await invoke(call, { channelId: this.channel.id });
      this.channel.favorite = !wasFav;
      this.toastr.success(msg);
    } catch (e) {
      this.error.handleError(e);
    }
  }

  async toggleHide() {
    if (!this.channel) return;
    const hide = !this.channel.hidden;
    try {
      await invoke("hide_channel", { id: this.channel.id, hidden: hide });
      this.channel.hidden = hide;
      this.toastr.success(
        `${hide ? "Hidden" : "Unhidden"} "${this.channel.name}"`,
      );
    } catch (e) {
      this.error.handleError(e);
    }
  }

  openRestream() {
    if (!this.channel) return;
    this.close();
    setTimeout(() => {
      this.memory.ModalRef = this.modal.open(RestreamModalComponent, {
        backdrop: "static",
        size: "xl",
        keyboard: false,
      });
      this.memory.ModalRef.componentInstance.channel = this.channel;
      this.memory.ModalRef.componentInstance.name = "RestreamModalComponent";
      this.memory.ModalRef.result.then(
        (_) => (this.memory.ModalRef = undefined),
      );
    }, 300);
  }

  openFullEPG() {
    if (!this.channel) return;
    this.close();
    setTimeout(async () => {
      try {
        const data: EPG[] = await invoke("get_epg", {
          channel: this.channel,
        });
        if (data.length === 0) {
          this.toastr.info("No EPG data for this channel");
          return;
        }
        this.memory.ModalRef = this.modal.open(EpgModalComponent, {
          backdrop: "static",
          size: "xl",
          keyboard: false,
        });
        this.memory.ModalRef.result.then(
          (_) => (this.memory.ModalRef = undefined),
        );
        this.memory.ModalRef.componentInstance.epg = data;
        this.memory.ModalRef.componentInstance.name = this.channel?.name;
        this.memory.ModalRef.componentInstance.channelId = this.channel?.id;
        this.memory.ModalRef.componentInstance.sourceId =
          this.channel?.source_id;
      } catch (e) {
        this.error.handleError(e, "Failed to load EPG data");
      }
    }, 300);
  }
}
