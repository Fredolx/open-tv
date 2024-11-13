import { Component } from '@angular/core';
import { NgbActiveModal } from '@ng-bootstrap/ng-bootstrap';
import { DialogFilter, open } from '@tauri-apps/plugin-dialog';
import { ErrorService } from '../error.service';
import { invoke } from '@tauri-apps/api/core';
import { MemoryService } from '../memory.service';

@Component({
  selector: 'app-import-modal',
  templateUrl: './import-modal.component.html',
  styleUrl: './import-modal.component.css'
})
export class ImportModalComponent {
  source_id?: number;
  nameOverride?: string;
  onlyPlaylists: boolean = false;
  constructor(public activeModal: NgbActiveModal, public memory: MemoryService) {

  }

  async selectFile() {
    let filters: DialogFilter[] = [
      {
        name: "Extension filter",
        extensions: this.onlyPlaylists ? ["otvp"] : ["otv", "otvg"]
      }
    ]
    const file = await open({
      multiple: false,
      directory: false,
      canCreateDirectories: false,
      title: "Select Open TV export file",
      filters: filters
    });
    if (file == null) {
      return;
    }
    this.nameOverride = this.nameOverride?.trim();
    if (this.nameOverride == "")
      this.nameOverride = undefined;
    let fail = await this.memory.tryIPC("Successfully imported file", "Failed to import file",
      () => invoke("import", { sourceId: this.source_id, path: file, nameOverride: this.nameOverride }));
    this.memory.RefreshSources.next(true);
    if (!fail)
      this.activeModal.close("close");
  }
}
