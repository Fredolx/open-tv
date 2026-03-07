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
import { MemoryService } from "../memory.service";
import { Channel } from "../models/channel";
import { MediaType } from "../models/mediaType";
import { PlayerEngine } from "../models/playerEngine";
import { PlayerState } from "../models/playerState";
import { ErrorService } from "../error.service";
import { Settings } from "../models/settings";
import { RecordingService } from "../recording.service";

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
  throughputDisplay: string | null = null;

  // Frost backdrop state
  frostPhase: 'none' | 'gradient' | 'frosted-frame' | 'revealing' = 'none';
  frostFrameUrl: string | null = null;
  private frostCanvas: HTMLCanvasElement | null = null;
  private frostCtx: CanvasRenderingContext2D | null = null;
  private frostRevealTimer: any = null;
  private frameCheckInterval: any = null;

  private subscriptions: Subscription[] = [];
  private controlsTimer: any = null;
  private dragStartY = 0;
  private dragThreshold = 150;
  private lastMouseMove = 0;

  // Stall detection: absorb minor dropouts silently
  private stallTimer: any = null;
  private stallGraceMs = 3500;
  private frostCooldownUntil = 0;
  private readonly FROST_COOLDOWN_MS = 800;
  private stallGeneration = 0;

  // Timeupdate throttle
  private lastTimeUpdateFlush = 0;

  // Stored listener refs for proper cleanup
  private onTimeUpdate: (() => void) | null = null;
  private onWaiting: (() => void) | null = null;
  private onPlaying: (() => void) | null = null;

  // Drag listener refs for cleanup on destroy
  private dragMoveHandler: ((e: MouseEvent | TouchEvent) => void) | null = null;
  private dragEndHandler: ((e: MouseEvent | TouchEvent) => void) | null = null;

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
    // Clean up leaked drag listeners before stopping playback
    if (this.dragMoveHandler) {
      document.removeEventListener('mousemove', this.dragMoveHandler);
      document.removeEventListener('touchmove', this.dragMoveHandler);
    }
    if (this.dragEndHandler) {
      document.removeEventListener('mouseup', this.dragEndHandler);
      document.removeEventListener('touchend', this.dragEndHandler);
    }
    this.dragMoveHandler = null;
    this.dragEndHandler = null;

    this.stopPlayback();
    this.subscriptions.forEach((s) => s.unsubscribe());
    this.clearControlsTimer();
    this.stopFrameDetection();
    this.clearFrostRevealTimer();
    this.cancelStallTimer();
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

  // ── Centralized overlay state manager ─────────────

  private updateOverlayState(state: 'loading' | 'stalling' | 'playing' | 'stopped') {
    switch (state) {
      case 'loading':
        this.loading = true;
        this.showFrostGradient();
        break;
      case 'stalling':
        this.loading = true;
        this.showFrostFrame();
        break;
      case 'playing':
        this.cancelStallTimer();
        this.loading = false;
        this.hideFrost();
        this.frostCooldownUntil = Date.now() + this.FROST_COOLDOWN_MS;
        break;
      case 'stopped':
        this.cancelStallTimer();
        this.loading = true;
        this.frostPhase = 'none';
        this.frostFrameUrl = null;
        break;
    }
  }

  // ── Playback lifecycle ────────────────────────────

  private async startPlayback() {
    if (!this.videoRef?.nativeElement || !this.channel) return;

    this.destroyPlayer();

    const video = this.videoRef.nativeElement;
    this.updateOverlayState('loading');
    this.startFrameDetection();

    try {
      const proxyUrl: string = await invoke("start_local_stream", { channel: this.channel });
      this.memory.LocalProxyRunning = true;
      this.attachPlayer(video, proxyUrl);
    } catch (e) {
      this.error.handleError(e, "Failed to start inline playback");
      this.stopPlayback();
    }
  }

  private attachPlayer(video: HTMLVideoElement, url: string) {
    this.destroyPlayer();

    video.src = url;
    video.volume = this.volume / 100;
    video.muted = this.isMuted;
    video.play().catch(() => {});
    this.isPlaying = true;

    // Video element event listeners
    this.removeVideoListeners();

    this.onTimeUpdate = () => {
      const now = Date.now();
      if (now - this.lastTimeUpdateFlush < 250) return;
      this.lastTimeUpdateFlush = now;
      this.zone.run(() => {
        this.currentTime = video.currentTime;
        this.duration = video.duration || 0;
      });
    };
    video.addEventListener("timeupdate", this.onTimeUpdate);

    this.onWaiting = () => {
      this.startStallTimer();
    };
    video.addEventListener("waiting", this.onWaiting);

    this.onPlaying = () => {
      if (this.loading || this.frostPhase !== 'none') {
        this.zone.run(() => this.updateOverlayState('playing'));
      } else {
        this.cancelStallTimer();
      }
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
  }

  private destroyPlayer() {
    this.removeVideoListeners();
    this.cancelStallTimer();
    this.clearFrostRevealTimer();
    this.throughputDisplay = null;
    const video = this.videoRef?.nativeElement;
    if (video) {
      video.pause();
      video.removeAttribute('src');
      video.load();
    }
  }

  async stopPlayback() {
    this.destroyPlayer();
    try { await invoke("stop_local_stream"); } catch (_) {}
    this.memory.LocalProxyRunning = false;
    this.updateOverlayState('stopped');
    this.stopFrameDetection();
    this.clearFrostRevealTimer();
    this.channel = null;
    this.isPlaying = false;
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
    this.isMuted = !this.isMuted;
    video.muted = this.isMuted;
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
      this.dragMoveHandler = null;
      this.dragEndHandler = null;
    };

    // Store refs so ngOnDestroy can clean up if component destroys mid-drag
    this.dragMoveHandler = onMove;
    this.dragEndHandler = onEnd;

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

  // ── Stall detection helpers ────────────────────────

  private startStallTimer() {
    if (this.stallTimer) return;
    if (Date.now() < this.frostCooldownUntil) return;
    const gen = ++this.stallGeneration;
    this.stallTimer = setTimeout(() => {
      this.stallTimer = null;
      if (gen !== this.stallGeneration) return;
      if (this.getBufferAhead() > 1) return;
      this.zone.run(() => this.updateOverlayState('stalling'));
    }, this.stallGraceMs);
  }

  private cancelStallTimer() {
    if (this.stallTimer) {
      clearTimeout(this.stallTimer);
      this.stallTimer = null;
    }
    this.stallGeneration++;
  }

  private getBufferAhead(): number {
    const video = this.videoRef?.nativeElement;
    if (!video || !video.buffered.length) return 0;
    const pos = video.currentTime;
    for (let i = 0; i < video.buffered.length; i++) {
      if (video.buffered.start(i) <= pos && video.buffered.end(i) > pos) {
        return video.buffered.end(i) - pos;
      }
    }
    return 0;
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
    this.clearFrostRevealTimer();
    const frame = this.captureVideoFrame();
    if (frame) {
      this.frostFrameUrl = frame;
      this.frostPhase = 'frosted-frame';
    } else if (this.frostPhase === 'none' || this.frostPhase === 'revealing') {
      this.showFrostGradient();
    }
  }

  private hideFrost() {
    if (this.frostPhase === 'none' || this.frostPhase === 'revealing') return;
    this.stopFrameDetection();
    this.frostPhase = 'revealing';
    this.clearFrostRevealTimer();
    this.frostRevealTimer = setTimeout(() => {
      if (this.frostPhase !== 'revealing') return;
      this.zone.run(() => {
        this.frostPhase = 'none';
        this.frostRevealTimer = null;
      });
    }, 420);
  }

  private clearFrostRevealTimer() {
    if (this.frostRevealTimer) {
      clearTimeout(this.frostRevealTimer);
      this.frostRevealTimer = null;
    }
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
