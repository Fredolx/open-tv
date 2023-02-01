import { Component, Input, OnInit } from '@angular/core';
import { invoke } from '@tauri-apps/api';
import { MemoryService } from '../memory.service';
import { Channel } from '../models/channel';

@Component({
  selector: 'app-channel-tile',
  templateUrl: './channel-tile.component.html',
  styleUrls: ['./channel-tile.component.css']
})
export class ChannelTileComponent implements OnInit {

  constructor(private memory: MemoryService) { }

  @Input() channel?: Channel;
  showImage: boolean = true;

  ngOnInit(): void {
  }

  async click(){
    await invoke("play_channel", {link: this.channel?.url});
    this.memory.CurrentChannel = this.channel?.url
  }

  isCurrentChannel(){
    return this.memory.CurrentChannel === this.channel?.url;
  }

  onError(event: Event) {
    this.showImage = false;
  }

}
