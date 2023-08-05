import { Component, Input, OnInit, ViewChild } from '@angular/core';
import { MatMenuTrigger } from '@angular/material/menu';
import { MemoryService } from '../memory.service';
import { Channel } from '../models/channel';
import { MediaType } from '../models/mediaType';

@Component({
  selector: 'app-channel-tile',
  templateUrl: './channel-tile.component.html',
  styleUrls: ['./channel-tile.component.scss']
})
export class ChannelTileComponent {
  constructor(public memory: MemoryService) { }
  @Input() channel?: Channel;
  @Input() id!: Number;
  @ViewChild(MatMenuTrigger, { static: true }) matMenuTrigger!: MatMenuTrigger;
  menuTopLeftPosition = { x: 0, y: 0 }
  showImage: boolean = true;
  starting: boolean = false;
  electron: any = (window as any).electronAPI;
  // less reactive but prevents the user from seeing the change in action
  alreadyExistsInFav = false;
  ngOnInit(): void {
    this.alreadyExistsInFav = this.alreadyExistsInFavorites();
  }

  async click(record = false) {
    if (this.channel?.type == MediaType.group) {
      this.memory.setCategoryNode(this.channel);
      return;
    }
    if (this.memory.StartingChannel)
      return;
    this.starting = true;
    this.memory.StartingChannel = true;
    this.electron.playChannel(this.channel?.url, record).then(() => {
    })
      .finally(() => {
        this.starting = false;
        this.memory.StartingChannel = false;
      });
  }

  onRightClick(event: MouseEvent) {
    if (this.channel?.type == MediaType.group)
      return;
    this.alreadyExistsInFav = this.alreadyExistsInFavorites();
    event.preventDefault();
    this.menuTopLeftPosition.x = event.clientX;
    this.menuTopLeftPosition.y = event.clientY;
    this.matMenuTrigger.openMenu();
  }

  onError(event: Event) {
    this.showImage = false;
  }

  favorite() {
    let index = this.memory.FavChannels.findIndex(x => x.url == this.channel?.url);
    if (index == -1) {
      this.memory.FavChannels.push(this.channel!);
    }
    else
      this.memory.FavChannels.splice(index, 1);
    this.memory.NeedToRefreshFavorites.next(true);
    this.electron.saveFavs(this.memory.FavChannels);
  }

  alreadyExistsInFavorites() {
    return this.memory.FavChannels.some(x => x.url == this.channel?.url);
  }

  async record() {
    await this.click(true);
  }

  isMovie() {
    return this.channel?.url?.endsWith('.mkv') || this.channel?.url?.endsWith('.mp4')
  }
}
