/**
 * Beats TV - Premium IPTV Player
 * Copyright (C) 2026 Beats TV Team
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along
 * with this program; if not, write to the Free Software Foundation, Inc.,
 * 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
 *
 * This project is a fork of Open TV by Fredolx.
 */

import { Injectable, NgZone } from "@angular/core";
import { Download } from "./models/download";
import { Subject } from "rxjs";
import { invoke } from "@tauri-apps/api/core";
import { ErrorService } from "./error.service";
import { listen } from "@tauri-apps/api/event";
import { Channel } from "./models/channel";

@Injectable({
  providedIn: "root",
})
export class DownloadService {
  Downloads: Map<String, Download> = new Map();

  constructor(
    private error: ErrorService,
    private ngZone: NgZone,
  ) { }

  async addDownload(id: string, channel: Channel): Promise<Download> {
    let download: Download = {
      channel: channel,
      progress: 0,
      complete: new Subject(),
      id: id,
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
      let download = this.Downloads.get(id);
      if (download) {
        await invoke("abort_download", {
          sourceId: download.channel.source_id,
          downloadId: download.id,
        });
        this.deleteDownload(download);
      }
    } catch (e) {
      console.error(e);
      this.error.handleError(e);
    }
  }

  async download(id: String, path?: string) {
    const download = this.Downloads.get(id);
    if (!download) {
      this.error.handleError(new Error("Download not found"), "Download not found");
      return;
    }
    try {
      await invoke("download", {
        downloadId: download.id,
        channel: download.channel,
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
