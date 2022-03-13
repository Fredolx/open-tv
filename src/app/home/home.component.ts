import { Component, OnInit } from '@angular/core';
import { Router } from '@angular/router';
import { MemoryService } from '../memory.service';

@Component({
  selector: 'app-home',
  templateUrl: './home.component.html',
  styleUrls: ['./home.component.css']
})
export class HomeComponent implements OnInit {

  constructor(private router: Router, private memory: MemoryService) {
    if(memory.Channels.length == 0)
      router.navigateByUrl("setup");
  }

  ngOnInit(): void {
  }

}
