import { Component, NgZone, OnDestroy, OnInit } from "@angular/core";
import { Channel } from "../models/channel";
import { invoke } from "@tauri-apps/api/core";
import { ErrorService } from "../error.service";
import { NetworkInfo } from "../models/networkInfo";
import { NgbActiveModal } from "@ng-bootstrap/ng-bootstrap";
import { UnlistenFn, listen } from "@tauri-apps/api/event";
import { save } from "@tauri-apps/plugin-dialog";
import { sanitizeFileName } from "../utils";
import { CHANNEL_EXTENSION } from "../models/extensions";

@Component({
  selector: "app-restream-modal",
  templateUrl: "./restream-modal.component.html",
  styleUrl: "./restream-modal.component.css",
})
export class RestreamModalComponent implements OnInit, OnDestroy {
  channel?: Channel;
  loading = false;
  watching = false;
  started = false;
  networkInfo?: NetworkInfo;
  selectedIP?: string;
  toUnlisten: UnlistenFn[] = [];

  constructor(
    private error: ErrorService,
    public activeModal: NgbActiveModal,
    private ngZone: NgZone,
  ) {}

  ngOnInit(): void {
    invoke("get_network_info").then((network) => {
      this.networkInfo = network as NetworkInfo;
      this.selectedIP = this.networkInfo.local_ips[0];
    });
    listen<boolean>("restream_started", () => {
      this.ngZone.run(() => {
        this.started = true;
        this.loading = false;
      });
    }).then((unlisten) => this.toUnlisten.push(unlisten));
  }

  async start() {
    this.loading = true;
    try {
      await invoke("start_restream", { channel: this.channel, port: this.networkInfo!.port });
    } catch (e) {
      this.error.handleError(e);
    }
    this.started = false;
    this.loading = false;
  }

  async stop() {
    this.loading = true;
    try {
      await invoke("stop_restream");
    } catch (e) {
      this.error.handleError(e);
    }
  }

  async watch() {
    this.watching = true;
    try {
      await invoke("watch_self", { port: this.networkInfo?.port });
    } catch (e) {
      this.error.handleError(e);
    }
    this.watching = false;
  }

  async share() {
    const file = await save({
      canCreateDirectories: true,
      title: "Select where to export re-stream",
      defaultPath: `${sanitizeFileName(this.channel?.name!)}_rst${CHANNEL_EXTENSION}`,
    });
    if (!file) {
      return;
    }
    try {
      await invoke("share_restream", {
        address: this.selectedIP,
        channel: this.channel,
        path: file,
      });
      this.error.success(`Successfully exported re-stream to ${file}`);
    } catch (e) {
      this.error.handleError(e);
    }
  }

  ngOnDestroy(): void {
    this.toUnlisten.forEach((x) => x());
  }
}
