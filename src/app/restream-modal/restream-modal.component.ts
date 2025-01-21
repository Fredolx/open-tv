import { Component } from "@angular/core";
import { Channel } from "../models/channel";
import { invoke } from "@tauri-apps/api/core";
import { ErrorService } from "../error.service";

@Component({
  selector: "app-restream-modal",
  templateUrl: "./restream-modal.component.html",
  styleUrl: "./restream-modal.component.css",
})
export class RestreamModalComponent {
  channel?: Channel;
  constructor(private error: ErrorService) {}
  async start() {
    try {
      await invoke("start_restream", { channel: this.channel });
    } catch (e) {
      this.error.handleError(e);
    }
  }

  async stop() {
    try {
      await invoke("stop_restream");
    } catch (e) {
      this.error.handleError(e);
    }
  }
}
