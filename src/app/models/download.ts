import { UnlistenFn } from "@tauri-apps/api/event";
import { Subject } from "rxjs";
import { Channel } from "./channel";

export enum DownloadStatus {
  queued = 0,
  downloading = 1,
}

export class Download {
  id!: string;
  progress!: number;
  complete!: Subject<boolean>;
  channel!: Channel;
  unlisten?: UnlistenFn;
  progressUpdate!: Subject<number>;
  status!: DownloadStatus;
  /// Where to save the file, picked before the download is queued
  path?: string;
  /// Resolves whatever called download(), once it finished, failed or got cancelled
  settle?: () => void;
}
