import { Component, Input } from '@angular/core';
import { Source } from '../../models/source';
import { SourceType } from '../../models/sourceType';

@Component({
  selector: 'app-source-tile',
  templateUrl: './source-tile.component.html',
  styleUrl: './source-tile.component.css'
})
export class SourceTileComponent {
  @Input("source")
  source?: Source;
  showUsername = false;
  showPassword = false;
  get_source_type_name() {
    if(!this.source)
      return null;
    return SourceType[this.source.source_type!];
  }
}
