import { Component, Input } from "@angular/core";
import { EPG } from "../../models/epg";
import { MemoryService } from "../../memory.service";
import { invoke } from "@tauri-apps/api/core";
import { EPGNotify } from "../../models/epgNotify";
import { ErrorService } from "../../error.service";
import { Channel } from "../../models/channel";
import { MediaType } from "../../models/mediaType";

@Component({
  selector: "app-epg-modal-item",
  templateUrl: "./epg-modal-item.component.html",
  styleUrl: "./epg-modal-item.component.css",
})
export class EpgModalItemComponent {
  constructor(
    public memory: MemoryService,
    private error: ErrorService,
  ) {}
  @Input()
  epg?: EPG;
  @Input()
  name?: string;
  playing: boolean = false;

  notificationOn(): boolean {
    return this.memory.Watched_epgs.has(this.epg!.epg_id);
  }

  async toggleNotification() {
    if (this.memory.LoadingNotification || !this.memory.trayEnabled) return;
    this.memory.LoadingNotification = true;
    if (!this.notificationOn()) {
      try {
        await invoke("add_epg", { epg: this.epg_to_epgNotify(this.epg!) });
        this.error.success("Added notification successfully");
      } catch (e) {
        this.error.handleError(e);
      }
    } else {
      try {
        await invoke("remove_epg", { epgId: this.epg?.epg_id });
        this.error.success("Removed notification successfully");
      } catch (e) {
        this.error.handleError(e);
      }
    }
    await this.memory.get_epg_ids();
    this.memory.LoadingNotification = false;
  }

  epg_to_epgNotify(epg: EPG): EPGNotify {
    return {
      channel_name: this.name!,
      epg_id: epg.epg_id,
      start_timestamp: epg.start_timestamp,
      title: epg.title,
    };
  }

  async timeshift() {
    if (this.playing) return;
    this.playing = true;
    let channel: Channel = {
      id: -1,
      url: this.epg?.timeshift_url,
      name: this.epg?.title,
      media_type: MediaType.movie,
      favorite: false,
    };
    try {
      await invoke("play", {
        channel: channel,
        record: false,
      });
    } catch (e) {
      console.error(e);
      this.error.handleError(e);
    } finally {
      this.playing = false;
    }
  }

  async download_timeshift() {
    await invoke("download", { name: this.name, url: this.epg?.timeshift_url });
  }
}
