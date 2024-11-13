import { Component } from '@angular/core';
import { NgbActiveModal } from '@ng-bootstrap/ng-bootstrap';
import { open } from '@tauri-apps/plugin-dialog';
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
  name?: string;
  constructor(public activeModal: NgbActiveModal, public memory: MemoryService) {

  }

  async selectFile() {
    const file = await open({
      multiple: false,
      directory: false
    });
    if (file == null) {
      return;
    }
    this.name = this.name?.trim();
    if (this.name == "")
      this.name = undefined;
    await this.memory.tryIPC("Successfully imported file", "Failed to import file",
       () => invoke("import", {sourceId: this.source_id, path: file, name: this.name}))
  }
}
