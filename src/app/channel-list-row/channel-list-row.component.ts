import { Component, OnInit } from "@angular/core";
import { ChannelTileComponent } from "../channel-tile/channel-tile.component";
import { MediaType } from "../models/mediaType";
import { invoke } from "@tauri-apps/api/core";

@Component({
  selector: "app-channel-list-row",
  templateUrl: "./channel-list-row.component.html",
  styleUrl: "./channel-list-row.component.css",
})
export class ChannelListRowComponent extends ChannelTileComponent implements OnInit {
  nowPlayingTitle = "";

  ngOnInit(): void {
    this.loadEPG();
  }

  async loadEPG() {
    if (this.channel?.media_type !== MediaType.livestream) return;
    try {
      const epgs = (await invoke("get_epg", { channel: this.channel })) as any[];
      const now = epgs?.find((e: any) => e.now_playing);
      if (now) this.nowPlayingTitle = now.title;
    } catch {
      // EPG not available for this channel
    }
  }

  getMediaTypeLabel(): string {
    switch (this.channel?.media_type) {
      case MediaType.livestream:
        return "LIVE";
      case MediaType.movie:
        return "MOVIE";
      case MediaType.serie:
        return "SERIES";
      case MediaType.group:
        return "GROUP";
      case MediaType.season:
        return "SEASON";
      default:
        return "";
    }
  }

  getMediaBadgeClass(): string {
    switch (this.channel?.media_type) {
      case MediaType.livestream:
        return "badge-live";
      case MediaType.movie:
        return "badge-movie";
      case MediaType.serie:
      case MediaType.season:
        return "badge-series";
      case MediaType.group:
        return "badge-group";
      default:
        return "";
    }
  }

  getGroupName(): string {
    return this.channel?.group_id ? `#${this.channel.group_id}` : "";
  }
}
