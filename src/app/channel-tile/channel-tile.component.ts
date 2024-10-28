import { Component, Input, ViewChild } from '@angular/core';
import { MatMenuTrigger } from '@angular/material/menu';
import { Channel } from '../models/channel';
import { MemoryService } from '../memory.service';
import { MediaType } from '../models/mediaType';
import { invoke } from '@tauri-apps/api/core';
import { ToastrService } from 'ngx-toastr';
import { ErrorService } from '../error.service';
import { NgbModal } from '@ng-bootstrap/ng-bootstrap';
import { EditChannelModalComponent } from '../edit-channel-modal/edit-channel-modal.component';
import { EditGroupModalComponent } from '../edit-group-modal/edit-group-modal.component';
import { DeleteGroupModalComponent } from '../delete-group-modal/delete-group-modal.component';

@Component({
  selector: 'app-channel-tile',
  templateUrl: './channel-tile.component.html',
  styleUrl: './channel-tile.component.css'
})
export class ChannelTileComponent {
  constructor(public memory: MemoryService, private toastr: ToastrService, private error: ErrorService, private modal: NgbModal) { }
  @Input() channel?: Channel;
  @Input() id!: Number;
  @ViewChild(MatMenuTrigger, { static: true }) matMenuTrigger!: MatMenuTrigger;
  menuTopLeftPosition = { x: 0, y: 0 }
  showImage: boolean = true;
  starting: boolean = false;
  // less reactive but prevents the user from seeing the change in action
  alreadyExistsInFav = false;
  mediaTypeEnum = MediaType;

  ngOnInit(): void {
  }

  async click(record = false) {
    if (this.starting === true)
      return;
    if (this.channel?.media_type == MediaType.group) {
      this.memory.SetGroupNode.next({ id: this.channel.id!, name: this.channel.name! });
      return;
    }
    if (this.channel?.media_type == MediaType.serie) {
      let id = Number.parseInt(this.channel.url!);
      if (!this.memory.SeriesRefreshed.has(id)) {
        this.memory.HideChannels.next(false);
        try {
          await invoke('get_episodes', { seriesId: id });
          this.memory.SeriesRefreshed.set(id, true);
        }
        catch (e) {
          this.error.handleError(e, "Failed to fetch series");
        }
      }
      this.memory.SetSeriesNode.next({ id: id, name: this.channel.name! });
      return;
    }
    this.starting = true;
    try {
      await invoke("play", { channel: this.channel, record: record });
    }
    catch (e) {
      this.error.handleError(e);
    }
    this.starting = false;
  }


  onRightClick(event: MouseEvent) {
    this.alreadyExistsInFav = this.channel!.favorite!;
    event.preventDefault();
    this.menuTopLeftPosition.x = event.clientX;
    this.menuTopLeftPosition.y = event.clientY;
    if (this.memory.currentContextMenu?.menuOpen)
      this.memory.currentContextMenu.closeMenu();
    this.memory.currentContextMenu = this.matMenuTrigger;
    this.matMenuTrigger.openMenu();
  }

  onError(event: Event) {
    this.showImage = false;
  }

  async favorite() {
    let call = 'favorite_channel';
    let msg = `Added ${this.channel?.name} to favorites`;
    if (this.channel!.favorite) {
      call = 'unfavorite_channel';
      msg = `Removed ${this.channel?.name} from favorites`
    }
    try {
      await invoke(call, { channelId: this.channel!.id });
      if (this.channel!.favorite)
        this.memory.Refresh.next(true);
      this.channel!.favorite = !this.channel!.favorite
      this.toastr.success(msg);
    }
    catch (e) {
      this.error.handleError(e, `Failed to add/remove ${this.channel?.name} to/from favorites`);
    }
  }

  async record() {
    await this.click(true);
  }

  isMovie() {
    return this.channel?.media_type != MediaType.livestream;
  }

  isCustomChannel(): boolean {
    return this.memory.CustomChannelsIds!.has(this.channel?.source_id!);
  }

  edit() {
    if (this.channel?.media_type == MediaType.group)
      this.edit_group();
    else {
      this.edit_channel();
    }
  }

  edit_group() {
    const modalRef = this.modal.open(EditGroupModalComponent, { backdrop: 'static', size: 'xl', });
    modalRef.componentInstance.name = "EditCustomGroupModal";
    modalRef.componentInstance.editing = true;
    modalRef.componentInstance.group = { id: this.channel!.id, name: this.channel!.name, image: this.channel!.image, source_id: this.channel!.source_id };
    modalRef.componentInstance.originalName = this.channel!.name;
  }

  edit_channel() {
    const modalRef = this.modal.open(EditChannelModalComponent, { backdrop: 'static', size: 'xl', });
    modalRef.componentInstance.name = "EditCustomChannelModal";
    modalRef.componentInstance.editing = true;
    modalRef.componentInstance.channel.data = { ...this.channel };
  }

  share() {
    if (this.channel?.media_type == MediaType.group) {
      this.memory.tryIPC(`Successfully exported category to Downloads/${this.channel?.id}.otvg`,
        "Failed to export channel",
        () => invoke('share_custom_group', { group: this.channel }));
    }
    else {
      this.memory.tryIPC(`Successfully exported channel to Downloads/${this.channel?.id}.otv`,
        "Failed to export channel",
        () => invoke('share_custom_channel', { channel: this.channel }));
    }
  }

  async delete() {
    if (this.channel?.media_type == MediaType.group)
      this.deleteGroup();
    else
      await this.deleteChannel();
  }

  async deleteGroup() {
    try {
      if (await invoke("group_not_empty", { id: this.channel?.id })) {
        this.openDeleteGroupModal();
      }
      else
        await this.deleteGroupNoReplace();
    }
    catch (e) {
      this.error.handleError(e);
    }
  }

  async deleteGroupNoReplace() {
    try {
      await invoke('delete_custom_group', { id: this.channel?.id, doChannelsUpdate: false });
      this.memory.Refresh.next(false);
      this.error.success("Successfully deleted category");
    }
    catch (e) {
      this.error.handleError(e);
    }
  }

  openDeleteGroupModal() {
    const modalRef = this.modal.open(DeleteGroupModalComponent, { backdrop: 'static', size: 'xl', });
    modalRef.componentInstance.name = "DeleteGroupModal";
    modalRef.componentInstance.group = { ...this.channel };
  }

  async deleteChannel() {
    await this.memory.tryIPC("Successfully deleted channel", "Failed to delete channel",
      () => invoke('delete_custom_channel', { id: this.channel?.id }))
    this.memory.Refresh.next(false);
  }
}
