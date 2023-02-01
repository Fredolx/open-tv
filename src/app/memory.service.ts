import { Injectable } from '@angular/core';
import { invoke } from '@tauri-apps/api';
import { ToastrService } from 'ngx-toastr';
import { Channel } from './models/channel';

@Injectable({
  providedIn: 'root'
})
export class MemoryService {

  constructor(private toast: ToastrService) { }
  private currentChannelInterval?: any;
  public Channels: Channel[] = [];
  private _currentChannel?: string;
  get CurrentChannel() {
    return this._currentChannel;
  }
  set CurrentChannel(value: string | undefined) {
    if (value) {
      if(this.currentChannelInterval != null)
        clearInterval(this.currentChannelInterval);
      this.currentChannelInterval = setInterval(() => this.checkIfThreadIsRunning(), 1000);
    }
    this._currentChannel = value;
  }

  checkIfThreadIsRunning() {
    invoke("any_threads_active").then(x => {
      if (x != undefined) {
        if(x != 0)
          this.toast.error("Could not play channel");
        this._currentChannel = undefined;
        clearInterval(this.currentChannelInterval);
        this.currentChannelInterval = null;
      }
    });
  }
}
