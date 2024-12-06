import { Component, OnInit } from '@angular/core';
import { NgbActiveModal, NgbTypeaheadSelectItemEvent } from '@ng-bootstrap/ng-bootstrap';
import { CustomChannel } from '../models/customChannel';
import { MediaType } from '../models/mediaType';
import { Channel, invoke } from '@tauri-apps/api/core';
import { MemoryService } from '../memory.service';
import { ChannelHeaders } from '../models/channelHeaders';
import { combineLatest, debounceTime, distinctUntilChanged, filter, from, map, Observable, OperatorFunction, Subject, Subscription, switchMap, tap } from 'rxjs';
import { IdName } from '../models/idName';
import { CustomChanelExtraData } from '../models/customChannelExtraData';
import { ErrorService } from '../error.service';

@Component({
  selector: 'app-edit-channel-modal',
  templateUrl: './edit-channel-modal.component.html',
  styleUrl: './edit-channel-modal.component.css'
})
export class EditChannelModalComponent implements OnInit {
  channel: CustomChannel = {
    data: {},
    headers: {}
  }
  beforeEditChannel?: CustomChannel;
  mediaTypeEnum = MediaType;
  editing: boolean = false;
  group?: IdName;
  search: any = (text$: Observable<string>) =>
    text$.pipe(
      debounceTime(200),
      distinctUntilChanged(),
      switchMap(term => {
        let promise: Promise<IdName> = invoke("group_auto_complete", { query: term, sourceId: this.channel.data.source_id });
        return from(promise);
      })
    );
  formatter = (result: IdName) => result.name;
  loading: boolean = false;
  channelExists = false;
  nameSubject = new Subject<string>();
  urlSubject = new Subject<string>();
  subscriptions: Subscription[] = [];

  constructor(public activeModal: NgbActiveModal, private memory: MemoryService, private error: ErrorService) {

  }

  onNameChange(val: string) {
    this.nameSubject.next(val);
  }

  onUrlChange(val: string) {
    this.urlSubject.next(val);
  }

  checkEmpty(val: IdName | string) {
    if (val == "") {
      this.channel.data.group_id = undefined;
    }
  }

  selectGroup(e: NgbTypeaheadSelectItemEvent) {
    this.channel.data.group_id = (e.item as IdName).id;
  }

  ngOnInit() {
    if (this.editing === true) {
      this.loading = true;
      try {
        invoke('get_custom_channel_extra_data', { id: this.channel.data.id, groupId: this.channel.data.group_id })
          .then(result => {
            let data = result as CustomChanelExtraData;
            if (data.headers)
              this.channel.headers = data.headers as ChannelHeaders
            if (data.group) {
              this.channel.data.group_id = data.group.id
              this.group = { id: data.group.id!, name: data.group.name! }
            }
          });
      }
      catch (e) {
        this.error.handleError(e);
      }
      this.loading = false;
    }
    combineLatest([this.nameSubject, this.urlSubject])
      .pipe(
        filter(([name, url]) => name != '' && name != null && url != '' && url != null),
        tap(() => this.loading = true),
        debounceTime(300),
      )
      .subscribe(([name, url]) => {
        this.channelExistsFn(name, url).then(() => this.loading = false);
    });
    this.beforeEditChannel =  JSON.parse(JSON.stringify(this.channel));
    this.nameSubject.next(this.channel.data.name!);
    this.urlSubject.next(this.channel.data.url!);
  }

  sanitize() {
    this.channel.data.name = this.channel.data.name?.trim();
    this.channel.data.image = this.channel.data.image?.trim();
    this.channel.data.url = this.channel.data.url?.trim();
    this.channel.headers!.http_origin = this.channel.headers?.http_origin?.trim();
    this.channel.headers!.user_agent = this.channel.headers?.user_agent?.trim();
    this.channel.headers!.referrer = this.channel.headers?.referrer?.trim();
  }

  async save() {
    this.loading = true;
    this.sanitize();
    let channel = { ...this.channel };
    channel.data.favorite = false;
    if (
      !channel.headers?.http_origin &&
      !channel.headers?.referrer &&
      !channel.headers?.user_agent &&
      !channel.headers?.ignore_ssl
    ) {
      channel.headers = undefined;
    }
    if (this.editing === true)
      await this.update_channel(channel);
    else
      await this.add_channel(channel);
    this.loading = false;
  }

  async update_channel(channel: CustomChannel) {
    try {
      await invoke("edit_custom_channel", { channel: channel });
      this.memory.Refresh.next(false);
      this.error.success("Successfully updated channel");
      this.activeModal.close('close');
    }
    catch (e) {
      this.error.handleError(e);
    }
  }

  async channelExistsFn(url: string, name: string) {
    this.channelExists = false;
    if (this.editing && this.beforeEditChannel?.data.name === this.channel.data.name && this.beforeEditChannel?.data.url === this.channel.data.url) {
      return;
    }
    this.channelExists = await invoke('channel_exists', { name: name, url: url, sourceId: this.channel.data.source_id }) as boolean;
  }

  async add_channel(channel: CustomChannel) {
    try {
      await invoke("add_custom_channel", { channel: channel });
      this.memory.RefreshSources.next(true);
      this.error.success("Successfully added channel");
      this.activeModal.close('close');
    }
    catch (e) {
      this.error.handleError(e);
    }
  }

  ngOnDestroy() {
    this.subscriptions.forEach(x => x.unsubscribe());
  }
}

