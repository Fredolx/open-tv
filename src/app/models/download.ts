import { UnlistenFn } from "@tauri-apps/api/event";
import { Subject } from "rxjs";
import { Channel } from "./channel";

export class Download {
  id!: string;
  progress!: number;
  complete!: Subject<boolean>;
  channel!: Channel;
  unlisten?: UnlistenFn;
  progressUpdate!: Subject<number>;
}
