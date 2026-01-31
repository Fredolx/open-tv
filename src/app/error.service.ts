/**
 * Beats TV - Premium IPTV Player
 * Copyright (C) 2026 Beats TV Team
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along
 * with this program; if not, write to the Free Software Foundation, Inc.,
 * 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
 *
 * This project is a fork of Open TV by Fredolx.
 */

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
