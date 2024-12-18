import { Injectable } from '@angular/core';
import { Source } from './models/source';
import { Subject } from 'rxjs';
import { MatMenuTrigger } from '@angular/material/menu';
import { IdName } from './models/idName';
import { ToastrService } from 'ngx-toastr';
import { ErrorService } from './error.service';
import { Channel } from './models/channel';
import { NgbModalRef } from '@ng-bootstrap/ng-bootstrap';

@Injectable({
  providedIn: 'root'
})
export class MemoryService {

  constructor(private toastr: ToastrService, private error: ErrorService) { }
  public SetGroupNode: Subject<IdName> = new Subject();
  public SetSeriesNode: Subject<Channel> = new Subject();
  public Sources: Source[] = [];
  public currentContextMenu?: MatMenuTrigger;
  public Loading = false;
  public Refresh: Subject<boolean> = new Subject();
  public RefreshSources: Subject<boolean> = new Subject();
  public AddingAdditionalSource = false;
  public SeriesRefreshed: Map<number, boolean> = new Map();
  public HideChannels: Subject<boolean> = new Subject();
  public CustomSourceIds: Set<number> = new Set();
  public XtreamSourceIds: Set<number> = new Set();
  public ModalRef?: NgbModalRef;
  async tryIPC<T>(
    successMessage: string,
    errorMessage: string,
    action: () => Promise<T>
  ): Promise<boolean> {
    this.Loading = true;
    let error = false;
    try {
      await action();
      this.toastr.success(successMessage);
    } catch (e) {
      this.error.handleError(e, errorMessage);
      error = true;
    }
    this.Loading = false;
    return error;
  }
}
