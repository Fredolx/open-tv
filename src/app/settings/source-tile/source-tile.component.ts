import { Component, Input } from '@angular/core';
import { Source } from '../../models/source';
import { SourceType } from '../../models/sourceType';
import { invoke } from '@tauri-apps/api/core';
import { ToastrService } from 'ngx-toastr';
import { MemoryService } from '../../memory.service';

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
  loading = false;
  constructor(public memory: MemoryService) {
  }

  get_source_type_name() {
    if(!this.source)
      return null;
    return SourceType[this.source.source_type!];
  }

  async refresh() {
    await this.memory.tryIPC("Successfully updated source", "Failed to refresh source", () => invoke("refresh_source", {source: this.source}));
  }

  async delete() {
    await this.memory.tryIPC("Successfully deleted source", "Failed to delete source", () => invoke("delete_source", {id: this.source?.id}));
    this.memory.RefreshSources.next(true);
  }
}
