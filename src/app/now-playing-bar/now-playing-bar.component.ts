import { Component, OnInit, OnDestroy } from "@angular/core";
import { trigger, style, transition, animate } from "@angular/animations";
import { Subscription } from "rxjs";
import { invoke } from "@tauri-apps/api/core";
import { MemoryService } from "../memory.service";
import { Channel } from "../models/channel";
import { MediaType } from "../models/mediaType";
import { PlayerEngine } from "../models/playerEngine";
import { PlayerState } from "../models/playerState";
import { ErrorService } from "../error.service";
import { RecordingService } from "../recording.service";
import { RecordingInfo } from "../models/recording";

@Component({
  selector: "app-now-playing-bar",
  templateUrl: "./now-playing-bar.component.html",
  styleUrl: "./now-playing-bar.component.css",
  animations: [
    trigger("barSlide", [
      transition(":enter", [
        style({ transform: "translateY(100%)" }),
        animate(
          "350ms cubic-bezier(0.16, 1, 0.3, 1)",
          style({ transform: "translateY(0)" }),
        ),
      ]),
      transition(":leave", [
        animate(
          "200ms ease-in",
          style({ transform: "translateY(100%)" }),
        ),
      ]),
    ]),
  ],
})
export class NowPlayingBarComponent implements OnInit, OnDestroy {
  channel: Channel | null = null;
  showImage = true;
  isWebPlayer = false;
  playerState: PlayerState = PlayerState.Closed;
  activeRecordings: RecordingInfo[] = [];
  currentTime = 0;

  private subscription?: Subscription;
  private engineSub?: Subscription;
  private stateSub?: Subscription;
  private recordingSub?: Subscription;
  private timeSub?: Subscription;

  constructor(
    public memory: MemoryService,
    private error: ErrorService,
    public recording: RecordingService,
  ) {}

  ngOnInit() {
    this.subscription = this.memory.NowPlaying.subscribe((ch) => {
      this.channel = ch;
      this.showImage = true;
    });
    this.engineSub = this.memory.PlayerEngine.subscribe((engine) => {
      this.isWebPlayer = engine === PlayerEngine.Web || engine === PlayerEngine.EmbeddedMpv;
    });
    this.stateSub = this.memory.PlayerState.subscribe((state) => {
      this.playerState = state;
    });
    this.recordingSub = this.recording.recordings$.subscribe((map) => {
      this.activeRecordings = Array.from(map.values());
    });
    this.timeSub = this.recording.currentTime$.subscribe((t) => {
      this.currentTime = t;
    });
  }

  ngOnDestroy() {
    this.subscription?.unsubscribe();
    this.engineSub?.unsubscribe();
    this.stateSub?.unsubscribe();
    this.recordingSub?.unsubscribe();
    this.timeSub?.unsubscribe();
  }

  isCurrentChannelRecording(): boolean {
    return !!this.channel?.id && this.recording.isRecording(this.channel.id);
  }

  stopRecording(recordingId: string) {
    this.recording.stopRecording(recordingId);
  }

  formatElapsed(startTs: number): string {
    return this.recording.formatElapsed(startTs, this.currentTime);
  }

  getSourceName(): string {
    if (!this.channel?.source_id) return "";
    return this.memory.Sources.get(this.channel.source_id)?.name || "";
  }

  isLive(): boolean {
    return this.channel?.media_type === MediaType.livestream;
  }

  async stop() {
    if (!this.channel) return;
    if (this.isWebPlayer) {
      // Web player: just clear NowPlaying; InlinePlayerComponent handles cleanup
      if (this.memory.LocalProxyRunning) {
        try { await invoke("stop_local_stream"); } catch (_) {}
        this.memory.LocalProxyRunning = false;
      }
    } else {
      try {
        await invoke("cancel_play", {
          sourceId: this.channel.source_id,
          channelId: this.channel.id,
        });
      } catch (e) {
        this.error.handleError(e);
      }
    }
    this.memory.NowPlaying.next(null);
  }

  expand() {
    this.memory.PlayerState.next(PlayerState.Expanded);
  }

  showInfo() {
    if (this.channel) {
      this.memory.ShowChannelDetail.next(this.channel);
    }
  }

  onImageError() {
    this.showImage = false;
  }
}
