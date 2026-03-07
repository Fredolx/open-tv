import {
  Component,
  ElementRef,
  OnDestroy,
  OnInit,
  ViewChild,
  HostListener,
  NgZone,
} from "@angular/core";
import { Subscription } from "rxjs";
import { invoke } from "@tauri-apps/api/core";
import Hls from "hls.js";
import { MemoryService } from "../memory.service";
import { Channel } from "../models/channel";
import { MediaType } from "../models/mediaType";
import { PlayerEngine } from "../models/playerEngine";
import { PlayerState } from "../models/playerState";
import { ErrorService } from "../error.service";
import { Settings } from "../models/settings";
import { RecordingService } from "../recording.service";

interface StreamInfo {
  url: string;
  has_custom_headers: boolean;
  is_hls: boolean;
}

@Component({
  selector: "app-inline-player",
  templateUrl: "./inline-player.component.html",
  styleUrl: "./inline-player.component.css",
})
export class InlinePlayerComponent implements OnInit, OnDestroy {
  @ViewChild("videoElement", { static: true }) videoRef!: ElementRef<HTMLVideoElement>;

  channel: Channel | null = null;
  playerState: PlayerState = PlayerState.Closed;
  isPlaying = false;
  isLive = true;
  isMuted = false;
  volume = 100;
  currentTime = 0;
  duration = 0;
  controlsVisible = true;
  isFullscreen = false;
  loading = true;
  isDragging = false;

  // Frost backdrop state
  frostPhase: 'none' | 'gradient' | 'frosted-frame' | 'revealing' = 'none';
  frostFrameUrl: string | null = null;
  private frostCanvas: HTMLCanvasElement | null = null;
  private frostCtx: CanvasRenderingContext2D | null = null;
  private frostRevealTimer: any = null;
  private frameCheckInterval: any = null;

  private hls: Hls | null = null;
  private subscriptions: Subscription[] = [];
  private controlsTimer: any = null;
  private usingProxy = false;
  private dragStartY = 0;
  private dragThreshold = 150;
  private lastMouseMove = 0;

  // Stored listener refs for proper cleanup
  private onTimeUpdate: (() => void) | null = null;
  private onWaiting: (() => void) | null = null;
  private onPlaying: (() => void) | null = null;
  private onLoadedMetadata: (() => void) | null = null;

  // Expose enum to template
  PlayerState = PlayerState;

  constructor(
    public memory: MemoryService,
    private error: ErrorService,
    private zone: NgZone,
    public recording: RecordingService,
  ) {}

  isCurrentChannelRecording(): boolean {
    return !!this.channel?.id && this.recording.isRecording(this.channel.id);
  }

  ngOnInit() {
    this.subscriptions.push(
      this.memory.NowPlaying.subscribe((ch) => {
        if (ch && this.memory.PlayerEngine.value === PlayerEngine.Web) {
          const isNewChannel = !this.channel || this.channel.id !== ch.id;
          this.channel = ch;
          this.isLive = ch.media_type === MediaType.livestream;
          this.memory.PlayerState.next(PlayerState.Expanded);
          if (isNewChannel) {
            this.loadSettings().then(() => this.startPlayback());
          }
        } else if (!ch) {
          this.stopPlayback();
        }
      }),
    );

    this.subscriptions.push(
      this.memory.PlayerState.subscribe((state) => {
        this.playerState = state;
      }),
    );
  }

  ngOnDestroy() {
    this.stopPlayback();
    this.subscriptions.forEach((s) => s.unsubscribe());
    this.clearControlsTimer();
    this.stopFrameDetection();
    this.frostCanvas = null;
    this.frostCtx = null;
  }

  @HostListener("document:keydown.escape")
  onEscapeKey() {
    if (this.playerState === PlayerState.Expanded) {
      this.minimize();
    }
  }

  private async loadSettings() {
    try {
      const settings = (await invoke("get_settings")) as Settings;
      this.volume = settings.volume ?? 100;
    } catch (_) {
      this.volume = 100;
    }
  }

  private async startPlayback() {
    if (!this.videoRef?.nativeElement || !this.channel) return;

    const video = this.videoRef.nativeElement;
    this.loading = true;
    this.showFrostGradient();
    this.startFrameDetection();

    try {
      const info: StreamInfo = await invoke("get_stream_info", { channel: this.channel });

      let playUrl = info.url;

      if (!info.is_hls || info.has_custom_headers) {
        playUrl = await invoke("start_local_stream", { channel: this.channel });
        this.usingProxy = true;
        this.memory.LocalProxyRunning = true;
        // Wait for ffmpeg to produce at least one segment
        await this.waitForManifest(playUrl);
      }

      this.attachHls(video, playUrl);
    } catch (e) {
      this.error.handleError(e, "Failed to start inline playback");
      this.stopPlayback();
    }
  }

  private async waitForManifest(url: string, timeoutMs = 15000): Promise<void> {
    const start = Date.now();
    while (Date.now() - start < timeoutMs) {
      try {
        const resp = await fetch(url);
        if (resp.ok) {
          const text = await resp.text();
          if (text.includes("#EXTINF")) return;
        }
      } catch (_) {}
      await new Promise((r) => setTimeout(r, 500));
    }
  }

  private attachHls(video: HTMLVideoElement, url: string) {
    this.destroyHls();

    if (Hls.isSupported()) {
      this.hls = new Hls({
        enableWorker: true,
        lowLatencyMode: this.isLive,
        maxBufferLength: this.isLive ? 30 : 60,
        maxMaxBufferLength: this.isLive ? 60 : 600,
        backBufferLength: this.isLive ? 30 : 60,
        liveSyncDurationCount: 3,
        liveMaxLatencyDurationCount: 10,
        fragLoadingMaxRetry: 6,
        fragLoadingRetryDelay: 1000,
        manifestLoadingMaxRetry: 4,
        manifestLoadingRetryDelay: 1000,
        levelLoadingMaxRetry: 4,
        levelLoadingRetryDelay: 1000,
        startFragPrefetch: true,
      });
      this.hls.loadSource(url);
      this.hls.attachMedia(video);
      this.hls.on(Hls.Events.MANIFEST_PARSED, () => {
        this.zone.run(() => {
          this.loading = false;
          this.hideFrost();
        });
        video.volume = this.volume / 100;
        video.play().catch(() => {});
        this.isPlaying = true;
      });
      this.hls.on(Hls.Events.ERROR, (_event: any, data: any) => {
        if (data.fatal) {
          switch (data.type) {
            case Hls.ErrorTypes.NETWORK_ERROR:
              this.hls?.startLoad();
              break;
            case Hls.ErrorTypes.MEDIA_ERROR:
              this.hls?.recoverMediaError();
              break;
            default:
              this.zone.run(() => {
                this.error.handleError(data.details, "Playback error");
                this.stopPlayback();
              });
              break;
          }
        }
      });
    } else if (video.canPlayType("application/vnd.apple.mpegurl")) {
      video.src = url;
      this.onLoadedMetadata = () => {
        this.loading = false;
        this.hideFrost();
        video.volume = this.volume / 100;
        video.play().catch(() => {});
        this.isPlaying = true;
      };
      video.addEventListener("loadedmetadata", this.onLoadedMetadata);
    }

    this.removeVideoListeners();

    this.onTimeUpdate = () => {
      this.zone.run(() => {
        this.currentTime = video.currentTime;
        this.duration = video.duration || 0;
      });
    };
    video.addEventListener("timeupdate", this.onTimeUpdate);

    this.onWaiting = () => {
      this.zone.run(() => {
        this.loading = true;
        this.showFrostFrame();
      });
    };
    video.addEventListener("waiting", this.onWaiting);

    this.onPlaying = () => {
      this.zone.run(() => {
        this.loading = false;
        this.hideFrost();
      });
    };
    video.addEventListener("playing", this.onPlaying);

    this.startControlsTimer();
  }

  private removeVideoListeners() {
    const video = this.videoRef?.nativeElement;
    if (!video) return;
    if (this.onTimeUpdate) { video.removeEventListener("timeupdate", this.onTimeUpdate); this.onTimeUpdate = null; }
    if (this.onWaiting) { video.removeEventListener("waiting", this.onWaiting); this.onWaiting = null; }
    if (this.onPlaying) { video.removeEventListener("playing", this.onPlaying); this.onPlaying = null; }
    if (this.onLoadedMetadata) { video.removeEventListener("loadedmetadata", this.onLoadedMetadata); this.onLoadedMetadata = null; }
  }

  private destroyHls() {
    this.removeVideoListeners();
    if (this.hls) {
      this.hls.destroy();
      this.hls = null;
    }
  }

  async stopPlayback() {
    this.destroyHls();
    if (this.usingProxy) {
      try { await invoke("stop_local_stream"); } catch (_) {}
      this.usingProxy = false;
      this.memory.LocalProxyRunning = false;
    }
    this.channel = null;
    this.isPlaying = false;
    this.loading = true;
    this.frostPhase = 'none';
    this.frostFrameUrl = null;
    this.stopFrameDetection();
    if (this.frostRevealTimer) {
      clearTimeout(this.frostRevealTimer);
      this.frostRevealTimer = null;
    }
    this.memory.PlayerState.next(PlayerState.Closed);
    this.clearControlsTimer();
  }

  close() {
    this.memory.NowPlaying.next(null);
  }

  togglePlay() {
    const video = this.videoRef?.nativeElement;
    if (!video) return;
    if (video.paused) {
      video.play().catch(() => {});
      this.isPlaying = true;
    } else {
      video.pause();
      this.isPlaying = false;
    }
  }

  onVolumeChange(event: Event) {
    const target = event.target as HTMLInputElement;
    this.volume = parseInt(target.value);
    const video = this.videoRef?.nativeElement;
    if (video) {
      video.volume = this.volume / 100;
      this.isMuted = this.volume === 0;
    }
  }

  toggleMute() {
    const video = this.videoRef?.nativeElement;
    if (!video) return;
    if (this.isMuted) {
      video.volume = this.volume / 100;
      this.isMuted = false;
    } else {
      video.volume = 0;
      this.isMuted = true;
    }
  }

  onSeek(event: Event) {
    const target = event.target as HTMLInputElement;
    const time = parseFloat(target.value);
    const video = this.videoRef?.nativeElement;
    if (video) {
      video.currentTime = time;
    }
  }

  minimize() {
    this.memory.PlayerState.next(PlayerState.Mini);
  }

  expand() {
    this.memory.PlayerState.next(PlayerState.Expanded);
  }

  onVideoClick(event: Event) {
    if (this.playerState === PlayerState.Mini) {
      event.stopPropagation();
      this.expand();
    }
  }

  toggleFullscreen() {
    const container = this.videoRef?.nativeElement?.closest('.player-container') as HTMLElement;
    if (!container) return;
    if (!document.fullscreenElement) {
      container.requestFullscreen().catch(() => {});
      this.isFullscreen = true;
    } else {
      document.exitFullscreen().catch(() => {});
      this.isFullscreen = false;
    }
  }

  @HostListener("document:fullscreenchange")
  onFullscreenChange() {
    this.isFullscreen = !!document.fullscreenElement;
  }

  onMouseMove() {
    if (this.playerState !== PlayerState.Expanded) return;
    const now = Date.now();
    if (now - this.lastMouseMove < 200) return;
    this.lastMouseMove = now;
    this.controlsVisible = true;
    this.startControlsTimer();
  }

  onDragStart(event: MouseEvent | TouchEvent) {
    if (this.playerState !== PlayerState.Expanded) return;
    event.preventDefault();
    this.isDragging = true;
    const clientY = event instanceof MouseEvent ? event.clientY : event.touches[0].clientY;
    this.dragStartY = clientY;

    const container = this.videoRef?.nativeElement?.closest('.player-container') as HTMLElement;

    const onMove = (e: MouseEvent | TouchEvent) => {
      const currentY = e instanceof MouseEvent ? e.clientY : e.touches[0].clientY;
      const delta = currentY - this.dragStartY;
      if (delta > 0 && container) {
        const clamped = Math.min(delta, 300);
        container.style.transform = `translateY(${clamped}px) scale(${Math.max(0.85, 1 - clamped / 1000)})`;
        container.style.opacity = `${Math.max(0.4, 1 - delta / 500)}`;
      }
    };

    const onEnd = (e: MouseEvent | TouchEvent) => {
      const endY = e instanceof MouseEvent ? e.clientY : (e as TouchEvent).changedTouches[0].clientY;
      const delta = endY - this.dragStartY;

      if (container) {
        container.style.transform = '';
        container.style.opacity = '';
      }

      this.zone.run(() => {
        if (delta > this.dragThreshold) {
          this.minimize();
        }
        this.isDragging = false;
      });

      document.removeEventListener('mousemove', onMove);
      document.removeEventListener('mouseup', onEnd);
      document.removeEventListener('touchmove', onMove);
      document.removeEventListener('touchend', onEnd);
    };

    document.addEventListener('mousemove', onMove);
    document.addEventListener('mouseup', onEnd);
    document.addEventListener('touchmove', onMove);
    document.addEventListener('touchend', onEnd);
  }

  private startControlsTimer() {
    this.clearControlsTimer();
    this.controlsTimer = setTimeout(() => {
      if (this.isPlaying && this.playerState === PlayerState.Expanded) {
        this.zone.run(() => {
          this.controlsVisible = false;
        });
      }
    }, 3000);
  }

  private clearControlsTimer() {
    if (this.controlsTimer) {
      clearTimeout(this.controlsTimer);
      this.controlsTimer = null;
    }
  }

  // ── Frost backdrop methods ──────────────────────────

  private initFrostCanvas() {
    if (this.frostCanvas) return;
    this.frostCanvas = document.createElement('canvas');
    this.frostCtx = this.frostCanvas.getContext('2d');
  }

  private captureVideoFrame(): string | null {
    const video = this.videoRef?.nativeElement;
    if (!video || video.videoWidth === 0) return null;
    this.initFrostCanvas();
    if (!this.frostCanvas || !this.frostCtx) return null;
    const w = Math.floor(video.videoWidth * 0.25);
    const h = Math.floor(video.videoHeight * 0.25);
    this.frostCanvas.width = w;
    this.frostCanvas.height = h;
    try {
      this.frostCtx.drawImage(video, 0, 0, w, h);
      return this.frostCanvas.toDataURL('image/jpeg', 0.5);
    } catch (_) {
      return null;
    }
  }

  private showFrostGradient() {
    this.frostFrameUrl = null;
    this.frostPhase = 'gradient';
  }

  private startFrameDetection() {
    this.stopFrameDetection();
    this.frameCheckInterval = setInterval(() => {
      const video = this.videoRef?.nativeElement;
      if (video && video.videoWidth > 0) {
        const frame = this.captureVideoFrame();
        if (frame) {
          this.zone.run(() => {
            this.frostFrameUrl = frame;
            this.frostPhase = 'frosted-frame';
          });
        }
        this.stopFrameDetection();
      }
    }, 100);
  }

  private stopFrameDetection() {
    if (this.frameCheckInterval) {
      clearInterval(this.frameCheckInterval);
      this.frameCheckInterval = null;
    }
  }

  private showFrostFrame() {
    const frame = this.captureVideoFrame();
    if (frame) {
      this.frostFrameUrl = frame;
      this.frostPhase = 'frosted-frame';
    } else {
      this.showFrostGradient();
    }
  }

  private hideFrost() {
    if (this.frostPhase === 'none') return;
    this.stopFrameDetection();
    this.frostPhase = 'revealing';
    if (this.frostRevealTimer) {
      clearTimeout(this.frostRevealTimer);
    }
    this.frostRevealTimer = setTimeout(() => {
      this.zone.run(() => {
        this.frostPhase = 'none';
        this.frostRevealTimer = null;
      });
    }, 420);
  }

  getSourceName(): string {
    if (!this.channel?.source_id) return "";
    return this.memory.Sources.get(this.channel.source_id)?.name || "";
  }

  formatTime(seconds: number): string {
    if (!isFinite(seconds) || seconds < 0) return "0:00";
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = Math.floor(seconds % 60);
    if (h > 0) {
      return `${h}:${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
    }
    return `${m}:${s.toString().padStart(2, "0")}`;
  }

  get seekPercent(): number {
    if (!this.duration || !isFinite(this.duration)) return 0;
    return (this.currentTime / this.duration) * 100;
  }

  get volumePercent(): number {
    return this.isMuted ? 0 : this.volume;
  }
}
