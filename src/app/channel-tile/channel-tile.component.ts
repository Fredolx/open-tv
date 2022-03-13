import { Component, Input, OnInit } from '@angular/core';
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

}
