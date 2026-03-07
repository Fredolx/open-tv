import { Component, ChangeDetectionStrategy } from "@angular/core";
import { ChannelTileComponent } from "../channel-tile/channel-tile.component";
import { MediaType } from "../models/mediaType";

@Component({
  selector: "app-channel-card",
  templateUrl: "./channel-card.component.html",
  styleUrl: "./channel-card.component.css",
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ChannelCardComponent extends ChannelTileComponent {
  // On dashboard cards, single click opens detail panel for playable channels.
  // Groups/series/seasons still drill in with the original behavior.
  override async click() {
    if (
      this.channel?.media_type === MediaType.serie ||
      this.channel?.media_type === MediaType.group ||
      this.channel?.media_type === MediaType.season
    ) {
      await super.click();
      return;
    }
    // Playable channels open the detail panel
    this.memory.ShowChannelDetail.next(this.channel!);
  }

  // Direct play bypasses the detail panel (used by the play overlay button)
  async playDirect() {
    await super.click();
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
}
