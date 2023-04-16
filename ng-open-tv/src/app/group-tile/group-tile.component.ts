import { Component, Input } from '@angular/core';
import { Channel } from '../models/channel';
import { MemoryService } from '../memory.service';

@Component({
  selector: 'app-group-tile',
  templateUrl: './group-tile.component.html',
  styleUrls: ['./group-tile.component.scss']
})
export class GroupTileComponent {
  @Input() channel?: Channel;
  menuTopLeftPosition = { x: 0, y: 0 }
  showImage: boolean = true;
  starting: boolean = false;

  constructor(private memory: MemoryService) {}

  click() {
    this.memory.SelectedCategory = this.channel;
    this.memory.CategoriesNode = this.memory.Channels.filter(x => x.group === this.channel?.group)
  }
}
