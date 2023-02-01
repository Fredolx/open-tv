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

  constructor(public memory: MemoryService) { }

  @Input() channel?: Channel;
  showImage: boolean = true;
  starting: boolean = false;
  ngOnInit(): void {
  }

  async click(){
    this.starting = true;
    this.memory.startingChannel = true;
    await invoke("play_channel", {link: this.channel?.url});    
    this.starting = false;
    this.memory.startingChannel = false;
  }


  onError(event: Event) {
    this.showImage = false;
  }

}
