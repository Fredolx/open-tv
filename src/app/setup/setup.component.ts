import { Component, OnInit } from '@angular/core';
import { invoke } from '@tauri-apps/api/tauri'
import { open, save } from "@tauri-apps/api/dialog"
import { MemoryService } from '../memory.service';
import { Channel } from '../models/channel';
import { Router } from '@angular/router';

@Component({
  selector: 'app-setup',
  templateUrl: './setup.component.html',
  styleUrls: ['./setup.component.css']
})
export class SetupComponent implements OnInit {

  constructor(public memory: MemoryService, private nav: Router) { }
  url?: string
  loading = false;
  ngOnInit(): void {
  }

  async getFile(){
    this.url = (await open() as string);
    if(!this.url.endsWith("m3u"))
      return;
    this.loading = true;
    this.memory.Channels = await invoke("get_playlist", {url: this.url});
    this.loading = false;
    this.nav.navigateByUrl("");
  }

}
