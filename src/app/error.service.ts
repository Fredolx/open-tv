import { Injectable } from "@angular/core";
import { NgbModal } from "@ng-bootstrap/ng-bootstrap";
import { ToastrService } from "ngx-toastr";
import { ErrorModalComponent } from "./error-modal/error-modal.component";
import { take } from "rxjs";
import { publicDir } from "@tauri-apps/api/path";

@Injectable({
  providedIn: "root",
})
export class ErrorService {
  constructor(
    private toastr: ToastrService,
    private modal: NgbModal,
  ) {}

  public handleError(e: any, message?: string) {
    let error = e as string;
    console.error(error);
    this.toastr
      .error(
        message
          ? message + ". Click here for more info"
          : "An error occured. Click here for more info",
      )
      .onTap.pipe(take(1))
      .subscribe(() => this.showError(error));
  }

  private showError(error: string) {
    const modalRef = this.modal.open(ErrorModalComponent, { backdrop: "static", size: "xl" });
    modalRef.componentInstance.name = "ErrorModal";
    modalRef.componentInstance.error = error;
  }

  public info(message: string) {
    this.toastr.info(message);
  }

  public success(message: string) {
    this.toastr.success(message);
  }
}
