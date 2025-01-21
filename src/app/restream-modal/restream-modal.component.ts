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
  loading = false;
  constructor(private error: ErrorService) {}
  async start() {
    this.loading = true;
    try {
      await invoke("start_restream", { channel: this.channel });
      this.error.success("Successfully started service");
    } catch (e) {
      this.error.handleError(e);
    }
    this.loading = false;
  }

  async stop() {
    this.loading = true;
    try {
      await invoke("stop_restream");
      this.error.success("Successfully stopped service");
    } catch (e) {
      this.error.handleError(e);
    }
    this.loading = false;
  }
}
