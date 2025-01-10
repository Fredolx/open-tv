import { Component, OnInit } from "@angular/core";
import { NgbActiveModal } from "@ng-bootstrap/ng-bootstrap";
import { EPG } from "../models/epg";
import { invoke } from "@tauri-apps/api/core";
import { MemoryService } from "../memory.service";

@Component({
  selector: "app-epg-modal",
  templateUrl: "./epg-modal.component.html",
  styleUrl: "./epg-modal.component.css",
})
export class EpgModalComponent implements OnInit {
  name?: string;
  epg: EPG[] = [];
  constructor(
    public activeModal: NgbActiveModal,
    private memory: MemoryService,
  ) {}
  ngOnInit() {
    invoke("get_epg_ids").then((x) => {
      let set = new Set(x as Array<string>);
      this.memory.Watched_epgs = set;
    });
  }
}
