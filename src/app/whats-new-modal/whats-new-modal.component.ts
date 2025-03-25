import { Component } from "@angular/core";
import { NgbActiveModal } from "@ng-bootstrap/ng-bootstrap";
import { MemoryService } from "../memory.service";

@Component({
  selector: "app-whats-new-modal",
  templateUrl: "./whats-new-modal.component.html",
  styleUrl: "./whats-new-modal.component.css",
})
export class WhatsNewModalComponent {
  constructor(
    private activeModal: NgbActiveModal,
    public memory: MemoryService,
  ) {}
  content?: string;

  close() {
    this.memory.updateVersion();
    this.activeModal.close("Cross click");
  }
}
