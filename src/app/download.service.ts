import { Injectable, NgZone } from "@angular/core";
import { Download, DownloadStatus } from "./models/download";
import { Subject } from "rxjs";
import { invoke } from "@tauri-apps/api/core";
import { ErrorService } from "./error.service";
import { listen } from "@tauri-apps/api/event";
import { Channel } from "./models/channel";

/// Used when a channel has no source id, so those downloads still queue against each other
const NO_SOURCE = -1;

@Injectable({
  providedIn: "root",
})
export class DownloadService {
  Downloads: Map<String, Download> = new Map();
  /// Downloads waiting for a free slot, in the order they were requested
  private queue: Download[] = [];
  /// The download currently running for a given source id
  private running: Map<number, Download> = new Map();

  constructor(
    private error: ErrorService,
    private ngZone: NgZone,
  ) { }

  async addDownload(id: string, channel: Channel): Promise<Download> {
    let existing = this.Downloads.get(id);
    if (existing) {
      return existing;
    }
    let download: Download = {
      channel: channel,
      progress: 0,
      complete: new Subject(),
      id: id,
      progressUpdate: new Subject(),
      status: DownloadStatus.queued,
    };

    download.unlisten = await listen<number>(`progress-${download.id}`, (event) => {
      this.ngZone.run(() => {
        download.progress = event.payload;
      });
      download.progressUpdate.next(download.progress);
    });
    this.Downloads.set(download.id, download);

    return download;
  }

  async abortDownload(id: String) {
    let download = this.Downloads.get(id);
    if (!download) {
      return;
    }
    // Nothing was ever sent to the backend, so there is no token to cancel
    if (download.status == DownloadStatus.queued) {
      this.queue = this.queue.filter((x) => x.id != download!.id);
      this.deleteDownload(download);
      download.settle?.();
      this.error.info("Download cancelled");
      return;
    }
    try {
      await invoke("abort_download", {
        sourceId: download.channel.source_id,
        downloadId: download.id,
      });
      // Cleanup and starting the next download is left to run(), whose invoke
      // is about to reject with "download aborted"
    } catch (e) {
      console.error(e);
      this.error.handleError(e);
    }
  }

  /// Queues a download and resolves once it completed, failed or got cancelled
  async download(id: String, path?: string) {
    let download = this.Downloads.get(id);
    if (!download) {
      return;
    }
    // Already waiting or running, don't queue the same channel twice
    if (this.queue.includes(download) || this.running.get(this.sourceKey(download)) == download) {
      return;
    }
    download.path = path;
    let settled = new Promise<void>((resolve) => (download!.settle = resolve));
    this.queue.push(download);
    this.startNext();
    return settled;
  }

  isQueued(download: Download) {
    return download.status == DownloadStatus.queued;
  }

  /// Starts every queued download whose source has no download running
  private startNext() {
    for (let download of [...this.queue]) {
      if (this.running.has(this.sourceKey(download))) {
        continue;
      }
      this.queue.splice(this.queue.indexOf(download), 1);
      this.running.set(this.sourceKey(download), download);
      this.run(download);
    }
  }

  private async run(download: Download) {
    this.ngZone.run(() => {
      download.status = DownloadStatus.downloading;
    });
    try {
      await invoke("download", {
        downloadId: download.id,
        channel: download.channel,
        path: download.path,
      });
      this.error.success("Download completed successfully");
    } catch (e) {
      if (e == "download aborted") this.error.info("Download cancelled");
      else this.error.handleError(e);
    }
    this.running.delete(this.sourceKey(download));
    this.deleteDownload(download);
    download.settle?.();
    this.startNext();
  }

  private sourceKey(download: Download) {
    return download.channel.source_id ?? NO_SOURCE;
  }

  deleteDownload(download: Download) {
    download.complete.next(true);
    try {
      download.unlisten!();
    } catch (e) {
      console.error(e);
    }
    this.Downloads.delete(download.id);
  }
}
