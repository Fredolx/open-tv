import { Injectable } from '@angular/core';
import { invoke } from '@tauri-apps/api';
import { ToastrService } from 'ngx-toastr';
import { Channel } from './models/channel';

@Injectable({
  providedIn: 'root'
})
export class MemoryService {

  constructor(private toast: ToastrService) { }
  public Channels: Channel[] = [];
  public startingChannel: boolean = false;
}
