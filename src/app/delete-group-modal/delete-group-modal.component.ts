import { Component } from '@angular/core';
import { NgbActiveModal, NgbTypeaheadSelectItemEvent } from '@ng-bootstrap/ng-bootstrap';
import { debounceTime, distinctUntilChanged, filter, from, map, Observable, switchMap } from 'rxjs';
import { IdName } from '../models/idName';
import { invoke } from '@tauri-apps/api/core';
import { Channel } from '../models/channel';
import { Group } from '../models/group';
import { ErrorService } from '../error.service';
import { MemoryService } from '../memory.service';

@Component({
  selector: 'app-delete-group-modal',
  templateUrl: './delete-group-modal.component.html',
  styleUrl: './delete-group-modal.component.css'
})
export class DeleteGroupModalComponent {
  loading: boolean = false;
  group?: Channel;
  new_group_id?: number;
  autocomplete_group?: IdName | string;
  search: any = (text$: Observable<string>) =>
    text$.pipe(
      debounceTime(200),
      distinctUntilChanged(),
      switchMap(term => {
        let promise: Promise<IdName[]> = invoke("group_auto_complete", { query: term, sourceId: this.group?.source_id });
        return from(promise).pipe(map(x => x.filter(y => y.id != this.group?.id)));
      }),
    );
  formatter = (result: IdName) => result.name;

  constructor(public activeModal: NgbActiveModal, private error: ErrorService, private memory: MemoryService) {

  }

  selectGroup(e: NgbTypeaheadSelectItemEvent) {
    this.new_group_id = (e.item as IdName).id;
  }

  checkEmpty(val: IdName | string) {
    if (val == "") {
      this.new_group_id = undefined;
    }
  }

  async delete() {
    this.loading = true;
    try {
      await invoke("delete_custom_group", { id: this.group?.id, newId: this.new_group_id, doChannelsUpdate: true });
      this.error.success("Successfully deleted category");
      this.memory.Refresh.next(false);
      this.activeModal.close("close");
    }
    catch (e) {
      this.error.handleError(e);
    }
    this.loading = false;
  }
}
