import { Component, Input } from '@angular/core';
import { Source } from '../../models/source';
import { SourceType } from '../../models/sourceType';
import { invoke } from '@tauri-apps/api/core';
import { MemoryService } from '../../memory.service';
import { EditChannelModalComponent } from '../../edit-channel-modal/edit-channel-modal.component';
import { NgbModal } from '@ng-bootstrap/ng-bootstrap';
import { EditGroupModalComponent } from '../../edit-group-modal/edit-group-modal.component';
import { ImportModalComponent } from '../../import-modal/import-modal.component';
import { open } from '@tauri-apps/plugin-dialog';

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
  sourceTypeEnum = SourceType;
  editing = false;
  editableSource: Source = {};

  constructor(public memory: MemoryService, private modal: NgbModal) {
  }

  get_source_type_name() {
    if (!this.source)
      return null;
    return SourceType[this.source.source_type!];
  }

  async refresh() {
    await this.memory.tryIPC("Successfully updated source", "Failed to refresh source", () => invoke("refresh_source", { source: this.source }));
  }

  async delete() {
    await this.memory.tryIPC("Successfully deleted source", "Failed to delete source", () => invoke("delete_source", { id: this.source?.id }));
    this.memory.RefreshSources.next(true);
  }

  async toggleEnabled() {
    await this.memory.tryIPC("Successfully toggled source", "Failed to toggle source", () => invoke("toggle_source", { value: !this.source?.enabled, sourceId: this.source?.id }));
    this.memory.RefreshSources.next(true);
  }

  async addCustomChannel() {
    const modalRef = this.modal.open(EditChannelModalComponent, { backdrop: 'static', size: 'xl', });
    modalRef.componentInstance.name = "EditCustomChannelModal";
    modalRef.componentInstance.channel.data.source_id = this.source?.id;
  }

  async addCustomGroup() {
    const modalRef = this.modal.open(EditGroupModalComponent, { backdrop: 'static', size: 'xl', });
    modalRef.componentInstance.name = "EditCustomGroupModal";
    modalRef.componentInstance.group.source_id = this.source?.id;
  }

  async import() {
    const modalRef = this.modal.open(ImportModalComponent, { backdrop: 'static', size: 'xl', });
    modalRef.componentInstance.name = "ImportModalComponent";
    modalRef.componentInstance.source_id = this.source?.id;
  }

  async share() {
    await this.memory.tryIPC(
      `Successfully exported source in ~/Downloads/${this.source?.id}.otvp`,
      "Failed to export source",
      () => invoke("share_custom_source", { source: this.source })
    );
  }

  edit() {
    this.editableSource = { ...this.source };
    this.editing = true;
  }

  async save() {
    await this.memory.tryIPC("Successfully saved changes", "Failed to save changes", 
      async () => {
        await invoke("update_source", { source: this.editableSource });
        this.source = this.editableSource;
        this.editing = false;
        this.editableSource = {};
    });
  }

  async browse() {
    const file = await open({
      multiple: false,
      directory: false,
      title: "Select a new m3u file for source"
    });
    if (file) {
      this.editableSource.url = file;
    }
  }

  cancel() {
    this.editableSource = {};
    this.editing = false;
  }
}
