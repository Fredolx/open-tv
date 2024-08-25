import { Injectable } from '@angular/core';
import { Source } from './models/source';
import { Subject } from 'rxjs';
import { MatMenuTrigger } from '@angular/material/menu';
import { IdName } from './models/idName';

@Injectable({
  providedIn: 'root'
})
export class MemoryService {

  constructor() { }
  public SetGroupNode: Subject<IdName> = new Subject();
  public Sources: Source[] = [];
  public currentContextMenu?: MatMenuTrigger;
  public Loading = false;
}
