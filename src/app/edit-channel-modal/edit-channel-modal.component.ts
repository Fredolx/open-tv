import { Component } from '@angular/core';
import { NgbActiveModal } from '@ng-bootstrap/ng-bootstrap';
import { CustomChannel } from '../models/customChannel';
import { MediaType } from '../models/mediaType';
import { invoke } from '@tauri-apps/api/core';
import { MemoryService } from '../memory.service';
import { ChannelHeaders } from '../models/channelHeaders';

@Component({
  selector: 'app-edit-channel-modal',
  templateUrl: './edit-channel-modal.component.html',
  styleUrl: './edit-channel-modal.component.css'
})
export class EditChannelModalComponent {
  channel: CustomChannel = {
    data: {},
    headers: {}
  }
  mediaTypeEnum = MediaType;
  editing: boolean = false;
  constructor(public activeModal: NgbActiveModal, private memory: MemoryService) {

  }

  ngOnInit() {
    if (this.editing === true) {
      invoke('get_channel_headers', { id: this.channel.data.id })
        .then(headers => {
          if (headers)
            this.channel.headers = headers as ChannelHeaders
        });
    }
  }

  async save() {
    let channel = { ...this.channel };
    channel.data.favorite = true;
    if (
      !channel.headers?.http_origin &&
      !channel.headers?.referrer &&
      !channel.headers?.user_agent &&
      !channel.headers?.ignore_ssl
    ) {
      channel.headers = undefined;
    }
    if (this.editing === true) {
      await this.memory.tryIPC("Successfully edited custom channel", "Failed to edit custom channel", () => invoke("edit_custom_channel", { channel: channel }))
      this.memory.Refresh.next(false);
    }
    else {
      await this.memory.tryIPC("Successfully added custom channel", "Failed to add custom channel", () => invoke("add_custom_channel", { channel: channel }));
      this.memory.RefreshSources.next(true);
    }
    this.activeModal.close('close');
  }
}
