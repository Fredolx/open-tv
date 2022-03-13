import { Component, OnInit } from '@angular/core';
import { Router } from '@angular/router';
import { invoke } from '@tauri-apps/api/tauri';
import { MemoryService } from '../memory.service';
import { Channel } from '../models/channel';

@Component({
  selector: 'app-home',
  templateUrl: './home.component.html',
  styleUrls: ['./home.component.css']
})
export class HomeComponent implements OnInit {

  constructor(private router: Router, public memory: MemoryService) {
    invoke("get_cache").then(x => {
      if (x)
        this.memory.Channels = x as Channel[];
      if (memory.Channels?.length == 0)
        router.navigateByUrl("setup");
    });
  }

  ngOnInit(): void {
  }

}
