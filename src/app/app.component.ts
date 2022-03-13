import { Component } from '@angular/core';
import { invoke } from '@tauri-apps/api/tauri'
import { Channel } from './models/channel'

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.css']
})
export class AppComponent {
  title = 'open-tv';
  constructor() {
  }
}
