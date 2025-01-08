import { Component, Input } from "@angular/core";
import { EPG } from "../../models/epg";
import { MemoryService } from "../../memory.service";

@Component({
  selector: "app-epg-modal-item",
  templateUrl: "./epg-modal-item.component.html",
  styleUrl: "./epg-modal-item.component.css",
})
export class EpgModalItemComponent {
  constructor(private memory: MemoryService) {}
  @Input()
  epg?: EPG;

  notificationOn() {}
}
