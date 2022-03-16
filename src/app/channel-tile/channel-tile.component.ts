import { Component, Input, OnInit } from '@angular/core';
import { invoke } from '@tauri-apps/api';
import { Channel } from '../models/channel';

@Component({
  selector: 'app-channel-tile',
  templateUrl: './channel-tile.component.html',
  styleUrls: ['./channel-tile.component.css']
})
export class ChannelTileComponent implements OnInit {

  constructor() { }

  @Input() channel?: Channel;

  ngOnInit(): void {
  }

  async click(){
    await invoke("play_channel", {link: this.channel?.url});
  }

}
