import { Component, Input, ViewChild } from '@angular/core';
import { MatMenuTrigger } from '@angular/material/menu';
import { Channel } from '../models/channel';
import { MemoryService } from '../memory.service';
import { MediaType } from '../models/mediaType';
import { invoke } from '@tauri-apps/api/core';
import { ToastrService } from 'ngx-toastr';

@Component({
  selector: 'app-channel-tile',
  templateUrl: './channel-tile.component.html',
  styleUrl: './channel-tile.component.css'
})
export class ChannelTileComponent {
  constructor(public memory: MemoryService, private toastr: ToastrService) { }
  @Input() channel?: Channel;
  @Input() id!: Number;
  @ViewChild(MatMenuTrigger, { static: true }) matMenuTrigger!: MatMenuTrigger;
  menuTopLeftPosition = { x: 0, y: 0 }
  showImage: boolean = true;
  starting: boolean = false;
  // less reactive but prevents the user from seeing the change in action
  alreadyExistsInFav = false;
  ngOnInit(): void {
  }

  async click(record = false) {
    if (this.channel?.media_type == MediaType.group) {
      this.memory.SetGroupNode.next({ id: this.channel.id, name: this.channel.name });
      return;
    }
    this.starting = true;
    try {
      await invoke("play", { channel: this.channel, record: record });
    }
    catch (e) {
      console.error(e)
    }
    this.starting = false;
  }

  onRightClick(event: MouseEvent) {
    if (this.channel?.media_type == MediaType.group)
      return;
    this.alreadyExistsInFav = this.channel!.favorite;
    event.preventDefault();
    this.menuTopLeftPosition.x = event.clientX;
    this.menuTopLeftPosition.y = event.clientY;
    if (this.memory.currentContextMenu?.menuOpen)
      this.memory.currentContextMenu.closeMenu();
    this.memory.currentContextMenu = this.matMenuTrigger;
    this.matMenuTrigger.openMenu();
  }

  onError(event: Event) {
    this.showImage = false;
  }

  async favorite() {
    let call = 'favorite_channel';
    let msg = `Added ${this.channel?.name} to favorites`;
    if (this.channel!.favorite) {
      call = 'unfavorite_channel';
      msg = `Removed ${this.channel?.name} from favorites`
    }
    try {
      await invoke(call, { channelId: this.channel!.id });
      if (this.channel!.favorite)
        this.memory.RefreshFavs.next(true);
      this.channel!.favorite = !this.channel!.favorite
      this.toastr.success(msg);
    }
    catch (e) {
      console.error(e);
      this.toastr.error(`Failed to add/remove ${this.channel?.name} to/from favorites`);
    }
  }

  async record() {
    await this.click(true);
  }

  isMovie() {
    return this.channel?.media_type != MediaType.livestream;
  }
}