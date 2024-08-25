import { Component, Input, ViewChild } from '@angular/core';
import { MatMenuTrigger } from '@angular/material/menu';
import { Channel } from '../models/channel';
import { MemoryService } from '../memory.service';
import { MediaType } from '../models/mediaType';
import { invoke } from '@tauri-apps/api/core';

@Component({
  selector: 'app-channel-tile',
  templateUrl: './channel-tile.component.html',
  styleUrl: './channel-tile.component.css'
})
export class ChannelTileComponent {
  constructor(public memory: MemoryService) { }
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
       this.memory.SetGroupNode.next({id: this.channel.id, name: this.channel.name});
       return;
    }
    this.starting = true;
    try {
      await invoke("play", {channel: this.channel, record: record});
    }
    catch(e) {
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

  favorite() {
    
  }

  async record() {
    await this.click(true);
  }

  isMovie() {
    return this.channel?.media_type != MediaType.livestream;
  }
}
