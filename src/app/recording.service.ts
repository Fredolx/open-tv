import { Injectable, OnDestroy } from "@angular/core";
import { BehaviorSubject } from "rxjs";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { ToastrService } from "ngx-toastr";
import { ErrorService } from "./error.service";
import { RecordingInfo } from "./models/recording";
import { Channel } from "./models/channel";

@Injectable({
  providedIn: "root",
})
export class RecordingService implements OnDestroy {
  recordings$ = new BehaviorSubject<Map<string, RecordingInfo>>(new Map());
  currentTime$ = new BehaviorSubject<number>(Date.now() / 1000);
  private busyChannels = new Set<number>();

  private timerInterval: ReturnType<typeof setInterval> | null = null;
  private unlistenStarted?: UnlistenFn;
  private unlistenStopped?: UnlistenFn;

  constructor(
    private toastr: ToastrService,
    private error: ErrorService,
  ) {
    this.initListeners();
    this.recoverState();
  }

  private async initListeners() {
    this.unlistenStarted = await listen<RecordingInfo>("recording-started", (event) => {
      const info = event.payload;
      const map = new Map(this.recordings$.value);
      map.set(info.recording_id, info);
      this.recordings$.next(map);
      this.toastr.success(`Recording started: ${info.channel_name}`);
      this.ensureTimer();
    });

    this.unlistenStopped = await listen<RecordingInfo>("recording-stopped", (event) => {
      const info = event.payload;
      const map = new Map(this.recordings$.value);
      map.delete(info.recording_id);
      this.recordings$.next(map);
      const filename = info.file_path.split(/[\\/]/).pop() ?? info.file_path;
      this.toastr.success(`Recording saved: ${filename}`);
      if (map.size === 0) this.stopTimer();
    });
  }

  private async recoverState() {
    try {
      const recordings = await invoke<RecordingInfo[]>("get_active_recordings");
      if (recordings.length > 0) {
        const map = new Map<string, RecordingInfo>();
        for (const r of recordings) {
          map.set(r.recording_id, r);
        }
        this.recordings$.next(map);
        this.ensureTimer();
      }
    } catch (_) {}
  }

  private ensureTimer() {
    if (this.timerInterval) return;
    this.timerInterval = setInterval(() => {
      this.currentTime$.next(Date.now() / 1000);
    }, 1000);
  }

  private stopTimer() {
    if (this.timerInterval) {
      clearInterval(this.timerInterval);
      this.timerInterval = null;
    }
  }

  isBusy(channelId: number): boolean {
    return this.busyChannels.has(channelId);
  }

  async startRecording(channel: Channel) {
    try {
      await invoke<RecordingInfo>("start_recording", { channel });
    } catch (e) {
      this.error.handleError(e, "Failed to start recording");
    }
  }

  async stopRecording(recordingId: string) {
    const rec = this.recordings$.value.get(recordingId);
    if (rec && this.busyChannels.has(rec.channel_id)) return;
    if (rec) this.busyChannels.add(rec.channel_id);
    try {
      await invoke<string>("stop_recording", { recordingId });
    } catch (e) {
      this.error.handleError(e, "Failed to stop recording");
    } finally {
      if (rec) this.busyChannels.delete(rec.channel_id);
    }
  }

  isRecording(channelId: number): boolean {
    for (const r of this.recordings$.value.values()) {
      if (r.channel_id === channelId) return true;
    }
    return false;
  }

  getRecordingForChannel(channelId: number): RecordingInfo | undefined {
    for (const r of this.recordings$.value.values()) {
      if (r.channel_id === channelId) return r;
    }
    return undefined;
  }

  async toggleRecording(channel: Channel) {
    if (!channel.id || this.busyChannels.has(channel.id)) return;
    this.busyChannels.add(channel.id);
    try {
      const existing = this.getRecordingForChannel(channel.id);
      if (existing) {
        await this.stopRecording(existing.recording_id);
      } else {
        await this.startRecording(channel);
      }
    } finally {
      this.busyChannels.delete(channel.id!);
    }
  }

  formatElapsed(startTimestamp: number, now: number): string {
    const elapsed = Math.max(0, Math.floor(now - startTimestamp));
    const h = Math.floor(elapsed / 3600);
    const m = Math.floor((elapsed % 3600) / 60);
    const s = elapsed % 60;
    if (h > 0) {
      return `${h}:${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
    }
    return `${m}:${s.toString().padStart(2, "0")}`;
  }

  ngOnDestroy() {
    this.stopTimer();
    this.unlistenStarted?.();
    this.unlistenStopped?.();
  }
}
