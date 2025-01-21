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
  started = false;
  address = "http://192.168.2.10/stream.m3u8";
  wanAddress = "http://10.145.22.12/stream.m3u8";
  constructor(private error: ErrorService) {}
  async start() {
    this.loading = true;
    try {
      await invoke("start_restream", { channel: this.channel });
      this.error.success("Successfully started service");
      this.started = true;
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
      this.started = false;
    } catch (e) {
      this.error.handleError(e);
    }
    this.loading = false;
  }

  async watch() {
    this.loading = true;
    try {
      await invoke("watch_self");
    } catch (e) {
      this.error.handleError(e);
    }
    this.loading = false;
  }

  share() {}
}
