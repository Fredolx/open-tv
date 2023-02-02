import { Component, Input, OnInit } from '@angular/core';
import { MemoryService } from '../memory.service';
import { Channel } from '../models/channel';

@Component({
  selector: 'app-channel-tile',
  templateUrl: './channel-tile.component.html',
  styleUrls: ['./channel-tile.component.scss']
})
export class ChannelTileComponent {
  constructor(public memory: MemoryService) { }
  @Input() channel?: Channel;
  showImage: boolean = true;
  starting: boolean = false;
  electron: any = (window as any).electronAPI;
  ngOnInit(): void {
  }

  async click(){
    this.starting = true;
    this.memory.startingChannel = true;
    this.electron.playChannel(this.channel?.url).then(() => {
    })
    .finally(() => {
      this.starting = false;
      this.memory.startingChannel = false;
    });
  }


  onError(event: Event) {
    this.showImage = false;
  }
}
