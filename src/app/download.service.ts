import { Injectable, NgZone } from "@angular/core";
import { Download } from "./models/download";
import { Subject } from "rxjs";
import { invoke } from "@tauri-apps/api/core";
import { ErrorService } from "./error.service";
import { listen } from "@tauri-apps/api/event";

@Injectable({
  providedIn: "root",
})
export class DownloadService {
  Downloads: Map<String, Download> = new Map();

  constructor(
    private error: ErrorService,
    private ngZone: NgZone,
  ) {}

  async addDownload(id: string, name: string, url: string): Promise<Download> {
    let download: Download = {
      name: name,
      progress: 0,
      complete: new Subject(),
      id: id,
      url: url,
      progressUpdate: new Subject(),
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
    try {
      await invoke("abort_download", {
        downloadId: id,
      });
      let download = this.Downloads.get(id);
      if (download) this.deleteDownload(download);
    } catch (e) {
      console.error(e);
      this.error.handleError(e);
    }
  }

  async download(id: String, path?: string) {
    let download = this.Downloads.get(id)!;
    try {
      await invoke("download", {
        downloadId: download.id,
        url: download.url,
        name: download.name,
        path: path,
      });
      this.error.success("Download completed successfully");
    } catch (e) {
      if (e == "download aborted") this.error.info("Download cancelled");
      else this.error.handleError(e);
    }
    this.deleteDownload(download);
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
