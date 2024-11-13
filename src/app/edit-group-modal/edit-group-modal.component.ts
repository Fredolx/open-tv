import { Component, OnInit } from '@angular/core';
import { NgbActiveModal } from '@ng-bootstrap/ng-bootstrap';
import { Group } from '../models/group';
import { ErrorService } from '../error.service';
import { invoke } from '@tauri-apps/api/core';
import { MemoryService } from '../memory.service';

@Component({
  selector: 'app-edit-group-modal',
  templateUrl: './edit-group-modal.component.html',
  styleUrl: './edit-group-modal.component.css'
})
export class EditGroupModalComponent {
  editing: boolean = false;
  group: Group = {};
  loading = false;
  originalName?: string;

  constructor(public activeModal: NgbActiveModal, private error: ErrorService, private memory: MemoryService) {

  }

  save() {
    this.loading = true;
    if (this.editing)
      this.edit_group();
    else
      this.add_group();
    this.loading = false;
  }

  sanitize() {
    this.group.name = this.group.name?.trim();
    this.group.image = this.group.image?.trim();
  }

  async edit_group() {
    try {
      this.sanitize();
      await invoke("edit_custom_group", { group: this.group });
      this.error.success("Successfully updated category");
      this.memory.Refresh.next(false);
      this.activeModal.close("close");
    }
    catch (e) {
      this.error.handleError(e);
    }
  }

  async add_group() {
    try {
      this.sanitize();
      await invoke("add_custom_group", { group: this.group })
      this.error.success("Successfully added category");
      this.memory.RefreshSources.next(true);
      this.activeModal.close("close");
    }
    catch (e) {
      this.error.handleError(e);
    }
  }
}
