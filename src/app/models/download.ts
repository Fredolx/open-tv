import { UnlistenFn } from "@tauri-apps/api/event";
import { Subject } from "rxjs";

export class Download {
  id!: string;
  progress!: number;
  complete!: Subject<boolean>;
  name!: string;
  url!: string;
  unlisten?: UnlistenFn;
  progressUpdate!: Subject<number>;
}
