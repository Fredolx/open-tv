import { Injectable } from '@angular/core';
import { Source } from './models/source';
import { Subject } from 'rxjs';
import { MatMenuTrigger } from '@angular/material/menu';
import { IdName } from './models/idName';
import { ToastrService } from 'ngx-toastr';

@Injectable({
  providedIn: 'root'
})
export class MemoryService {

  constructor(private toastr: ToastrService ) { }
  public SetGroupNode: Subject<IdName> = new Subject();
  public SetSeriesNode: Subject<IdName> = new Subject();
  public Sources: Source[] = [];
  public currentContextMenu?: MatMenuTrigger;
  public Loading = false;
  public RefreshFavs: Subject<boolean> = new Subject();
  public RefreshSources: Subject<boolean> = new Subject();
  public AddingAdditionalSource = false;
  public SeriesRefreshed: Map<Number, boolean> = new Map();
  public HideChannels: Subject<boolean> = new Subject();

  async tryIPC<T>(
    successMessage: string,
    errorMessage: string,
    action: () => Promise<T>
  ): Promise<void> {
    this.Loading = true;
    try {
      await action();
      this.toastr.success(successMessage);
    } catch (e) {
      console.error(e);
      this.toastr.error(errorMessage);
    }
    this.Loading = false;
  }
}
