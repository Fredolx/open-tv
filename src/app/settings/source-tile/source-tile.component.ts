import { Component, Input } from "@angular/core";
import { Source } from "../../models/source";
import { SourceType } from "../../models/sourceType";
import { invoke } from "@tauri-apps/api/core";
import { MemoryService } from "../../memory.service";
import { EditChannelModalComponent } from "../../edit-channel-modal/edit-channel-modal.component";
import { NgbModal } from "@ng-bootstrap/ng-bootstrap";
import { EditGroupModalComponent } from "../../edit-group-modal/edit-group-modal.component";
import { ImportModalComponent } from "../../import-modal/import-modal.component";
import { open, save } from "@tauri-apps/plugin-dialog";
import { CHANNEL_EXTENSION, FAVS_BACKUP, PLAYLIST_EXTENSION } from "../../models/extensions";
import { sanitizeFileName } from "../../utils";

@Component({
  selector: "app-source-tile",
  templateUrl: "./source-tile.component.html",
  styleUrl: "./source-tile.component.css",
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
  defaultUserAgent = "Fred TV";

  constructor(
    public memory: MemoryService,
    private modal: NgbModal,
  ) {}

  get_source_type_name() {
    if (!this.source) return null;
    return SourceType[this.source.source_type!];
  }

  async refresh() {
    if (this.source?.source_type == SourceType.Xtream) this.memory.SeriesRefreshed.clear();
    await this.memory.tryIPC("Successfully updated source", "Failed to refresh source", () =>
      invoke("refresh_source", { source: this.source }),
    );
  }

  async delete() {
    await this.memory.tryIPC("Successfully deleted source", "Failed to delete source", () =>
      invoke("delete_source", { id: this.source?.id }),
    );
    this.memory.RefreshSources.next(true);
  }

  async toggleEnabled() {
    await this.memory.tryIPC("Successfully toggled source", "Failed to toggle source", () =>
      invoke("toggle_source", { value: !this.source?.enabled, sourceId: this.source?.id }),
    );
    this.memory.RefreshSources.next(true);
  }

  async addCustomChannel() {
    this.memory.ModalRef = this.modal.open(EditChannelModalComponent, {
      backdrop: "static",
      size: "xl",
      keyboard: false,
    });
    this.memory.ModalRef.result.then((_) => (this.memory.ModalRef = undefined));
    this.memory.ModalRef.componentInstance.name = "EditCustomChannelModal";
    this.memory.ModalRef.componentInstance.channel.data.source_id = this.source?.id;
  }

  async addCustomGroup() {
    this.memory.ModalRef = this.modal.open(EditGroupModalComponent, {
      backdrop: "static",
      size: "xl",
      keyboard: false,
    });
    this.memory.ModalRef.result.then((_) => (this.memory.ModalRef = undefined));
    this.memory.ModalRef.componentInstance.name = "EditCustomGroupModal";
    this.memory.ModalRef.componentInstance.group.source_id = this.source?.id;
  }

  async import() {
    this.memory.ModalRef = this.modal.open(ImportModalComponent, {
      backdrop: "static",
      size: "xl",
      keyboard: false,
    });
    this.memory.ModalRef.result.then((_) => (this.memory.ModalRef = undefined));
    this.memory.ModalRef.componentInstance.name = "ImportModalComponent";
    this.memory.ModalRef.componentInstance.source_id = this.source?.id;
  }

  async share() {
    let file = await save({
      canCreateDirectories: true,
      title: "Select where to export custom source",
      defaultPath: sanitizeFileName(this.source?.name!) + PLAYLIST_EXTENSION,
    });
    if (file) {
      await this.memory.tryIPC(
        `Successfully exported source in ${file}`,
        "Failed to export source",
        () => invoke("share_custom_source", { source: this.source, path: file }),
      );
    }
  }

  edit() {
    this.editableSource = { ...this.source };
    this.editing = true;
  }

  async save() {
    await this.memory.tryIPC("Successfully saved changes", "Failed to save changes", async () => {
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
      title: "Select a new m3u file for source",
    });
    if (file) {
      this.editableSource.url = file;
    }
  }

  cancel() {
    this.editableSource = {};
    this.editing = false;
  }

  async backupFavs() {
    const file = await save({
      canCreateDirectories: true,
      title: "Select where to save favorites",
      defaultPath: `${sanitizeFileName(this.source?.name!)}_favs${FAVS_BACKUP}`,
    });
    if (file) {
      await this.memory.tryIPC(
        "Successfully saved favorites backup",
        "Failed to save favorites backup",
        async () => {
          await invoke("backup_favs", { id: this.source?.id, path: file });
        },
      );
    }
  }

  async restoreFavs() {
    const file = await open({
      canCreateDirectories: false,
      title: "Select a favorites backup",
      directory: false,
      multiple: false,
      filters: [{ name: "extension", extensions: ["otvf"] }],
    });
    if (file) {
      await this.memory.tryIPC(
        "Successfully saved favorites backup",
        "Failed to save favorites backup",
        async () => {
          await invoke("restore_favs", { id: this.source?.id, path: file });
        },
      );
    }
  }
}
